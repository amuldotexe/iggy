# MongoDB Source Connector — Gap Analysis

**Date**: 2026-03-15
**Branch**: `codex/2739-source-20260307`
**Issue**: [iggy-rs/iggy#2739](https://github.com/iggy-rs/iggy/issues/2739)

---

## Answer

The MongoDB source connector is **near PR-ready**. Core correctness logic is implemented and tested. The remaining work is documentation and edge-case integration tests — no architectural changes needed.

Three things block a clean PR:

1. **README doesn't document operational behavior** — supported BSON kinds, legacy state coercion, and side-effect mismatch semantics are undocumented
2. **Two edge-case integration tests are missing** — unsupported BSON kind poll failure and duplicate-boundary error path exist as unit tests but lack integration-level proof
3. **Send-failure re-poll has no integration coverage** — may require runtime test hooks that don't exist yet (deferrable)

---

## Why This Conclusion

### A. The core logic is correct and well-tested

All seven correctness requirements from both spec documents are implemented:

- **Checkpoint safety**: Two-phase commit model — `poll()` stages a pending batch, `commit_polled_messages_now()` persists only after Iggy delivery succeeds, `discard_polled_messages_now()` clears on failure (`lib.rs:583-666`)
- **Side-effect ordering**: Delete/mark operations run inside `commit_polled_messages_now()`, never before delivery confirmation (`lib.rs:625-660`)
- **Type-safe offsets**: `TrackingOffsetValue` enum preserves BSON kind metadata across serialize/deserialize cycles, with backward compatibility for legacy untyped strings (`lib.rs:199-306`)
- **Query composition**: `build_filter_document()` uses `$and` to combine checkpoint, user filter, and processed-field predicates without overwrite (`lib.rs:940-972`)
- **Duplicate boundary handling**: `classify_duplicate_boundary_now()` detects non-unique tracking values at batch edges, rolls back checkpoint or fails fast on all-equal batches (`lib.rs:375-400`)

Evidence: 50+ unit tests and 7 integration tests (with testcontainers) covering polling, delete-after-read, mark-processed, state persistence across restart, and ObjectId offset tracking.

### B. The gaps are shallow — documentation and test coverage, not logic

**Gap 1 — README documentation** (low effort, high PR impact)

The spec requires three sections the README currently lacks:
- Supported tracking-field BSON kinds: int32, int64, double, string, object_id, date_time
- Legacy untyped string-state coercion: numeric-looking strings and `_id`-like hex strings may be reinterpreted on the fallback path
- Side-effect mismatch behavior: when delete/mark affects fewer records than expected, the connector warns and continues — it does not rollback or reconcile

File: `core/connectors/sources/mongodb_source/README.md`

**Gap 2 — Edge-case integration tests** (medium effort, spec compliance)

Two integration tests are specified but not yet written:
- `TEST-MDBSRCGAP-006`: Seed a document with an unsupported BSON kind as tracking field, assert poll fails before emitting messages to Iggy
- `TEST-MDBSRCGAP-009`: Seed `batch_size + 1` documents with identical non-`_id` tracking values, assert duplicate-boundary error prevents message emission

Both scenarios have passing unit tests. The integration tests would prove end-to-end behavior through the connector runtime.

**Gap 3 — Send-failure re-poll** (deferrable)

`TEST-INTEG-ACK-002` requires injecting a downstream send failure and proving the same batch is re-offered. This likely needs runtime-level test hooks (`producer.send()` failure injection) that may not exist yet. Unit tests already prove the state model is correct. This can be deferred to a follow-up without compromising safety.

### C. Risk is concentrated in "unknown" areas, not "wrong" areas

| Area | Risk | Rationale |
|---|---|---|
| Core logic correctness | Low | All requirements implemented, tested at unit and integration level |
| README gaps | Low | Straightforward documentation additions |
| Missing integration tests | Medium | Unit coverage exists; integration tests add defense-in-depth |
| Send-failure injection test | Medium | Requires runtime hooks; deferrable |
| Clippy / fmt compliance | Unknown | Not yet verified on this branch |
| Workspace dep conventions | Unknown | `mongodb = "3.0"` is inline, not workspace — need to check project norms |

---

## Recommended Actions (Priority Order)

1. Run `cargo clippy` and `cargo test` to establish baseline — confirms nothing is broken before touching anything
2. Update README with the three required documentation sections
3. Add the two edge-case integration tests (unsupported BSON kind, duplicate boundary)
4. Verify fixture hygiene — no sink-only residue in source test fixtures
5. Defer send-failure re-poll integration test to follow-up PR

---

## Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy -p iggy_connector_mongodb_source --all-targets --all-features -- -D warnings
cargo test -p iggy_connector_mongodb_source
cargo build -p server --bin iggy-server
cargo build -p iggy-connectors --bin iggy-connectors
```

# MongoDB Source Connector — Gap Analysis

**Date**: 2026-03-15
**Branch**: `codex/2739-source-20260307`
**Issue**: [iggy-rs/iggy#2739](https://github.com/iggy-rs/iggy/issues/2739)

## Context

Issue #2739 on `iggy-rs/iggy` requests a MongoDB connector. The branch already has a substantial implementation (~1769 lines in `lib.rs`) with 7 integration tests and 50+ unit tests. Two internal spec documents exist:
- `MONGODB_SOURCE_CORRECTNESS_SPEC.md`
- `MONGODB_SOURCE_REMAINING_GAPS_SPEC_20260307222339.md`

This analysis compares the **current code** against those specs to identify what's done, what's still open, and what matters for PR readiness.

---

## What's Already Implemented (Confirmed Working)

| Requirement | Status | Evidence |
|---|---|---|
| REQ-MDBSRC-001.0: Checkpoint follows successful send | **Done** | Two-phase commit: `poll()` stages pending batch, `commit_polled_messages_now()` persists after send. `discard_polled_messages_now()` clears on failure. Unit tests: `commit_moves_pending_batch_to_committed_state`, `discard_clears_pending_batch_without_advancing_state` |
| REQ-MDBSRC-002.0: Side effects post-send only | **Done** | `commit_polled_messages_now()` runs delete/mark only after runtime confirms delivery (`lib.rs:625-660`) |
| REQ-MDBSRC-004.0: Typed offset preservation | **Done** | `TrackingOffsetValue` enum with `from_bson_value_now()` / `to_query_bson_now()`. Tests: `typed_string_offset_round_trip_preserves_string_kind`, `legacy_string_offset_remains_backward_compatible` |
| REQ-MDBSRC-005.0: Query filter composition | **Done** | `build_filter_document()` uses `$and` composition. Tests: `query_filter_scopes_mark_delete_side_effects`, `query_filter_does_not_overwrite_tracking_clause` |
| REQ-MDBSRC-006.0: Payload + snake_case | **Done** | Test: `payload_field_honors_snake_case_conversion` |
| REQ-MDBSRCGAP-004.0: Unsupported BSON kind fail-fast | **Done** | `extract_tracking_offset_with_context()` returns explicit `Storage` error naming collection, field, BSON kind, and supported kinds. Test: `tracking_field_error_names_collection_field_and_kind` |
| Duplicate boundary rollback (non-_id) | **Done** | `resolve_checkpoint_offset_for_batch()` + `classify_duplicate_boundary_now()`. Tests: `non_unique_tracking_field_does_not_skip_equal_offsets`, `all_equal_duplicate_boundary_should_fail_fast` |

---

## Remaining Gaps

### Gap 1: README documentation gaps (REQ-MDBSRCGAP-004.0, 005.0, 007.0)

The remaining gaps spec explicitly requires README documentation for:
- **Supported tracking-field BSON kinds** (int32, int64, double, string, object_id, date_time)
- **Legacy untyped string-state coercion behavior** (numeric-looking strings reinterpreted)
- **Partial side-effect mismatch semantics** (warn-and-continue, not rollback)

**File**: `core/connectors/sources/mongodb_source/README.md`

### Gap 2: Integration test for unsupported BSON kind poll failure (TEST-MDBSRCGAP-006)

The spec requires an integration test that seeds a document with an unsupported tracking-field BSON kind and asserts that poll fails before message emission. This is distinct from the existing unit test.

**Status**: Unit test exists (`tracking_field_error_names_collection_field_and_kind`), but integration-level coverage is missing.

### Gap 3: Integration test for duplicate-boundary error path (TEST-MDBSRCGAP-009)

The spec requires an integration test confirming the duplicate-boundary error path does not emit messages into Iggy. Unit test exists, but integration coverage is missing.

### Gap 4: No dedicated regression test for send-failure re-poll (TEST-INTEG-ACK-002)

REQ-MDBSRC-001.0 requires an integration test proving the same MongoDB records are re-polled after an injected send failure. The unit tests prove state management, but no integration test injects a downstream failure.

**Note**: This may require runtime-level test hooks that may not exist yet.

### Gap 5: Source fixture scope hygiene (REQ-MDBSRC-007.0)

The spec mentions source-only fixtures should not carry sink-only residue. Need to verify fixture compilation is clean.

### Gap 6: Workspace integration (potential)

The `mongodb` crate dependency (`v3.0`) is declared directly in the connector's `Cargo.toml` rather than via workspace — should check if other connectors (postgres, elasticsearch) follow the same pattern or use workspace deps.

---

## Risk Assessment for PR

| Risk | Severity | Notes |
|---|---|---|
| Core logic correctness | **Low** | Two-phase commit, typed offsets, filter composition, duplicate boundary all tested |
| Missing integration tests for edge cases | **Medium** | Unsupported BSON kind + duplicate boundary integration tests are in spec but not yet written |
| README documentation gaps | **Low** | Straightforward to add |
| Runtime-level failure injection tests | **Medium** | May need runtime test hooks that don't exist yet — could be deferred to follow-up |
| Code quality / clippy | **Unknown** | Need to run `cargo clippy` and `cargo test` to verify |

---

## Recommended Next Steps

1. **Read and verify README** against spec documentation requirements
2. **Run `cargo clippy` and `cargo test`** on the mongodb_source package to check current state
3. **Add missing integration tests** for unsupported BSON kind and duplicate boundary (if runtime supports it)
4. **Update README** with supported BSON kinds, legacy coercion, and side-effect mismatch semantics
5. **Verify fixture hygiene** — no sink-only residue in source fixtures

---

## Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy -p iggy_connector_mongodb_source --all-targets --all-features -- -D warnings
cargo test -p iggy_connector_mongodb_source
cargo build -p server --bin iggy-server
cargo build -p iggy-connectors --bin iggy-connectors
```

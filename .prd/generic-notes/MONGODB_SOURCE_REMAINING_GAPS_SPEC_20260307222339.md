# MongoDB Source Remaining Gaps Spec

## Context

This spec corrects the stale parts of [MONGODB_SOURCE_CORRECTNESS_SPEC.md](/Users/amuldotexe/Desktop/iggy202603070600/iggy/.prd/generic-notes/MONGODB_SOURCE_CORRECTNESS_SPEC.md).

It is based on the current branch state of the MongoDB source connector and on the corrected findings below:

- The source delivery boundary is currently correct: runtime send happens before commit, and failed send discards the pending batch.
- Typed tracking offsets are currently correct: `TrackingOffsetValue` plus `PersistedTrackingOffset` preserve BSON kind for new state.
- `query_filter` composition is currently correct and already has targeted unit tests.
- Duplicate-boundary fail-fast behavior is currently implemented, but the all-equal error path lacks a dedicated regression test.
- The remaining correctness gaps are not typed-offset corruption or query-filter overwrite. The real remaining work is:
  - explicit validation and documentation for unsupported tracking-field BSON kinds
  - clearer behavior and documentation for legacy untyped string-state coercion
  - explicit regression coverage for the all-equal duplicate-boundary error path
  - honest documentation for partial side-effect mismatch behavior, which is currently warn-and-continue rather than rollback/reconcile

## Scope

This spec covers only the remaining MongoDB source hardening work that is still relevant on the current branch.

It does not reopen already-correct items unless the purpose is non-regression coverage.

## Actors And Boundaries

- Actor: MongoDB source connector
- Actor: connectors runtime source-delivery path
- Actor: persisted connector state loader
- Boundary: typed persisted state vs legacy untyped state
- Boundary: poll-time duplicate batch boundary handling
- Boundary: post-send delete/mark side effects vs committed checkpoint state
- Boundary: README and executable tests as the source of truth for behavior

## Failure Modes

- A legacy string offset such as `"42"` is reinterpreted by the backward-compatible fallback path.
- A future document uses a tracking-field BSON kind that the connector cannot safely compare or persist.
- More than `batch_size` records share the same non-`_id` tracking value, and the all-equal branch regresses silently.
- Docs or follow-up PRs overstate side-effect mismatch semantics as rollback/reconcile when the code only warns and commits.

## Reliability Limits

- No new performance claims are made in this spec.
- Backward compatibility for legacy state must be preserved unless a migration path is explicitly introduced.
- MongoDB is schema-less, so unsupported tracking-field kinds cannot always be rejected strictly at startup unless startup validation actively samples data.

## Executable Requirements

### REQ-MDBSRCGAP-001.0: Source delivery acknowledgment SHALL remain post-send only

**WHEN** the MongoDB source runtime poll returns a batch  
**THEN** the system SHALL keep checkpoint and side-effect state pending until the runtime successfully sends that batch to Iggy  
**AND** SHALL discard the pending batch on downstream send failure  
**SHALL** continue to re-offer the same logical batch after a failed send.

Notes:
- This requirement is already implemented.
- This requirement exists as a non-regression guard because the remaining-gap work must not disturb the current source acknowledgment boundary.

### REQ-MDBSRCGAP-002.0: Typed tracking offsets SHALL preserve BSON comparison kind

**WHEN** the connector persists and restores typed tracking offsets  
**THEN** the system SHALL preserve enough type information to rebuild the original BSON comparison kind  
**AND** SHALL preserve typed string offsets such as `"42"` as `Bson::String` during subsequent query construction  
**SHALL** keep backward compatibility for pre-existing legacy untyped string state.

Notes:
- This requirement is already implemented for typed state.
- Legacy untyped state is intentionally different and is covered separately below.

### REQ-MDBSRCGAP-003.0: Query filters SHALL continue to compose with checkpoint predicates

**WHEN** `query_filter` is configured together with tracking-field or processed-field predicates  
**THEN** the connector SHALL compose them with logical `AND` semantics  
**AND** SHALL NOT allow user filter content to overwrite the connector-generated checkpoint predicate  
**SHALL** keep the same predicate-composition behavior across poll, delete, and mark paths.

Notes:
- This requirement is already implemented and already has targeted unit tests.
- It remains in this spec to prevent stale reviews from reopening a solved issue.

### REQ-MDBSRCGAP-004.0: Unsupported tracking-field BSON kinds SHALL fail fast with explicit diagnostics

**WHEN** the connector can determine that a tracking field resolves to a BSON kind it does not support for extraction, persistence, or comparison  
**THEN** the system SHALL fail before acknowledging the batch  
**AND** SHALL return an explicit error naming:
- collection
- tracking field
- observed BSON kind
- supported BSON kinds
**SHALL** document the supported tracking-field BSON kinds in the README.

Clarification:
- Because MongoDB is schema-less, this may happen during startup validation if sampling is added, or at first observation during polling if no such sampling exists.
- This requirement does not assume startup rejection is always possible on an empty collection.

### REQ-MDBSRCGAP-005.0: Legacy untyped string state SHALL remain backward compatible and SHALL be documented honestly

**WHEN** the connector loads a legacy persisted tracking offset stored as an untyped string  
**THEN** the system SHALL keep backward-compatible fallback behavior for query conversion  
**AND** SHALL document that numeric-looking strings and `_id`-like strings may be reinterpreted by that legacy fallback path  
**SHALL** keep new persisted state type-tagged so that the ambiguity does not affect fresh checkpoints.

Clarification:
- This requirement is about compatibility and documentation, not about changing legacy behavior immediately.
- A future migration away from `LegacyString` is allowed only if there is an explicit compatibility plan.

### REQ-MDBSRCGAP-006.0: All-equal duplicate batch boundaries SHALL have explicit regression coverage

**WHEN** a non-`_id` tracking field produces `batch_size + 1` records with the same boundary value and no previous distinct value exists in the batch  
**THEN** the poll path SHALL fail fast with the duplicate-boundary error  
**AND** SHALL NOT emit produced messages for that poll  
**SHALL** have a dedicated regression test for that exact all-equal error path.

### REQ-MDBSRCGAP-007.0: Partial side-effect mismatches SHALL be documented as warn-and-continue

**WHEN** `delete_after_read` or `processed_field` side effects affect fewer records than the connector expected, but MongoDB still returns a successful operation result  
**THEN** the connector SHALL log a warning describing the mismatch  
**AND** SHALL continue to commit already-delivered checkpoint state as it does today  
**SHALL** document this behavior as observability-only mismatch handling, not rollback, retry, or side-effect reconciliation.

Clarification:
- This requirement aligns the spec with the current code path.
- It does not introduce a new rollback mechanism.

## Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-MDBSRCGAP-001.0 | TEST-MDBSRCGAP-001 | integration | failed downstream send leaves pending batch uncommitted and re-pollable | runtime boundary |
| REQ-MDBSRCGAP-002.0 | TEST-MDBSRCGAP-002 | unit | typed string `"42"` round-trips as typed string and rebuilds `Bson::String` | typed state |
| REQ-MDBSRCGAP-002.0 | TEST-MDBSRCGAP-003 | unit | legacy string `"42"` remains backward compatible and converts through fallback path | legacy state |
| REQ-MDBSRCGAP-003.0 | TEST-MDBSRCGAP-004 | unit | `query_filter` and tracking predicate compose via `$and` without overwrite | filter composition |
| REQ-MDBSRCGAP-004.0 | TEST-MDBSRCGAP-005 | unit | unsupported BSON kind returns explicit error with kind and supported-kind list | validation |
| REQ-MDBSRCGAP-004.0 | TEST-MDBSRCGAP-006 | integration | first observed unsupported tracking-field BSON kind aborts poll before message emission | poll failure |
| REQ-MDBSRCGAP-005.0 | TEST-MDBSRCGAP-007 | docs/unit | legacy-state behavior is preserved in code and documented in README | compatibility |
| REQ-MDBSRCGAP-006.0 | TEST-MDBSRCGAP-008 | unit | all-equal duplicate boundary returns duplicate-boundary error and no checkpoint | duplicate boundary |
| REQ-MDBSRCGAP-006.0 | TEST-MDBSRCGAP-009 | integration | duplicate-boundary error path does not emit messages into Iggy | regression coverage |
| REQ-MDBSRCGAP-007.0 | TEST-MDBSRCGAP-010 | unit/docs | mismatch classification still warns, and README describes warn-and-continue semantics | side-effect honesty |

## TDD Plan

### STUB

1. Add a dedicated unit test for the all-equal duplicate-boundary error path.
2. Add unit coverage for explicit unsupported-BSON-kind diagnostics.
3. Add an integration test that seeds a document with an unsupported tracking-field BSON kind and asserts that poll fails before emission.
4. Add README assertions or checklist items for:
   - supported BSON kinds
   - legacy untyped string coercion
   - warn-and-continue side-effect mismatch behavior

### RED

1. Run the existing MongoDB source unit tests.
2. Add the new tests and observe the current failures:
   - unsupported BSON kinds fail too generically
   - the exact all-equal duplicate-boundary branch has no dedicated regression test
   - README does not currently explain legacy coercion clearly enough
   - README does not currently state that partial side-effect mismatches only warn and continue

### GREEN

1. Introduce one shared helper for supported tracking-kind diagnostics.
2. Improve `InvalidRecord`-style failures on unsupported tracking-field kinds so the error includes field, collection, actual kind, and supported kinds.
3. Add the explicit all-equal duplicate-boundary test.
4. Update README sections for:
   - supported tracking-field BSON kinds
   - legacy untyped string-state behavior
   - partial side-effect mismatch semantics

### REFACTOR

1. Keep typed-state logic and legacy fallback logic clearly separated.
2. Keep duplicate-boundary decision logic in one helper so the error-path tests target one source of truth.
3. Prefer explicit helper names for validation and supported-kind formatting.

### VERIFY

1. `cargo fmt --all -- --check`
2. `cargo clippy -p iggy_connector_mongodb_source --all-targets --all-features -- -D warnings`
3. `cargo test -p iggy_connector_mongodb_source`
4. `cargo build -p server --bin iggy-server`
5. `cargo build -p iggy-connectors --bin iggy-connectors`
6. `cargo test -p integration --test mod -- mongodb_source`
7. Manual README review against every `REQ-MDBSRCGAP-*` behavior statement

## Quality Gates

- [ ] Source send-before-commit behavior remains unchanged.
- [ ] Typed tracking-offset round-trip tests remain green.
- [ ] Legacy string-state backward compatibility remains green.
- [ ] Unsupported BSON kind errors are explicit and name supported kinds.
- [ ] README lists supported tracking-field BSON kinds.
- [ ] README explains legacy string-state coercion behavior.
- [ ] README describes partial side-effect mismatches as warn-and-continue.
- [ ] Dedicated all-equal duplicate-boundary regression coverage exists.
- [ ] No stale spec or README text claims that typed string-state preservation is currently broken.
- [ ] No stale spec or README text claims that `query_filter` overwrite protection is missing.

## Open Questions

1. Should the connector add startup sampling to detect unsupported tracking-field BSON kinds before the first successful poll, or is fail-on-first-observation sufficient?
2. Should legacy untyped string-state loading emit a one-time warning, or is README documentation enough for the first hardening pass?
3. Should partial side-effect mismatches eventually become connector metrics in addition to warnings?
4. Do we want to explicitly support `Bson::Timestamp` in tracking fields later, or keep the supported set limited to the currently type-tagged kinds?

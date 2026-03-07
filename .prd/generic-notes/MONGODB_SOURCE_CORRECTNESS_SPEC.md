# MongoDB Source Correctness Hardening Spec

## Context

This spec is based on the current `codex/2739-source-20260307` delta against `master`, the MongoDB sink review feedback on PR `#2815`, and the maintainer notes captured in `.prd/generic-notes`.

Observed lessons carried forward from GitHub review and connector notes:

- Narrow scope matters. Runtime-wide behavior changes hidden inside a connector PR create review friction.
- Partial success paths must be modeled explicitly. A connector cannot report success for work that was only partly durable.
- Delivery semantics must match the real runtime boundary, not just the connector-local state mutation order.
- Comments and tests need to prove invariants, not just restate intent.

## Executable Requirements

### REQ-MDBSRC-001.0: Checkpoint advancement SHALL follow successful downstream send

**WHEN** a MongoDB source poll returns messages to the connectors runtime  
**THEN** the system SHALL keep the batch checkpoint pending until the runtime successfully sends that batch to Iggy  
**AND** SHALL NOT advance the live in-memory checkpoint on `producer.send()` failure  
**SHALL** re-offer the same batch on the next poll after a send failure

### REQ-MDBSRC-002.0: Source-side mark or delete SHALL not run before send acknowledgment

**WHEN** `delete_after_read` or `processed_field` mode is enabled  
**THEN** the system SHALL execute MongoDB delete/mark side effects only after the runtime confirms the batch was sent successfully  
**AND** SHALL preserve the source records unchanged when send, encode, or state-save fails before acknowledgment  
**SHALL** reject these modes at startup if the runtime cannot provide a post-send acknowledgment hook

### REQ-MDBSRC-003.0: Non-unique tracking fields SHALL not stall or replay forever

**WHEN** more than `batch_size` documents share the same tracking-field value at the batch boundary  
**THEN** the system SHALL either process them with a stable tie-breaker or fail fast with a typed configuration/runtime error  
**AND** SHALL NOT return a batch while leaving checkpoint, mark/delete, and retry position unchanged  
**SHALL** have an automated test covering the “all records in batch share the same offset” case

### REQ-MDBSRC-004.0: Tracking offsets SHALL preserve BSON type semantics

**WHEN** the connector persists and restores tracking offsets  
**THEN** the system SHALL retain enough type information to rebuild the original MongoDB comparison type for the tracking field  
**AND** SHALL NOT coerce string-backed offsets such as `"42"` into numeric BSON for subsequent queries  
**SHALL** reject unsupported tracking-field BSON types with an explicit configuration/runtime error and documentation entry

### REQ-MDBSRC-005.0: Query filters SHALL compose without weakening checkpoint predicates

**WHEN** a user configures `query_filter` and the connector also needs tracking or processed predicates  
**THEN** the system SHALL combine them with logical `AND` semantics  
**AND** SHALL preserve the connector-generated checkpoint predicate even if `query_filter` mentions the same field  
**SHALL** use the same effective predicate for poll, mark, and delete paths

### REQ-MDBSRC-006.0: Payload extraction SHALL remain stable under field-name normalization

**WHEN** `snake_case_fields = true` and `payload_field` is configured  
**THEN** the system SHALL define one consistent lookup rule for `payload_field`  
**AND** SHALL document whether lookup happens before or after key normalization  
**SHALL** have a unit test covering camelCase payload extraction with snake-case conversion enabled

### REQ-MDBSRC-007.0: Source-only test fixtures SHALL not carry sink-only residue

**WHEN** the source connector integration fixtures are compiled  
**THEN** the test crate SHALL build without dead sink-only constants or source-unrelated warnings in MongoDB source fixture modules  
**AND** SHALL keep source-only fixture exports and environment variables limited to the source test surface  
**SHALL** avoid copying sink-only setup artifacts into the source PR delta

## Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-MDBSRC-001.0 | TEST-UNIT-ACK-001 | unit | failed downstream send leaves live checkpoint unchanged | correctness |
| REQ-MDBSRC-001.0 | TEST-INTEG-ACK-002 | integration | same MongoDB records are re-polled after injected send failure | delivery semantics |
| REQ-MDBSRC-002.0 | TEST-INTEG-SIDEFX-003 | integration | delete/mark occurs only after successful send acknowledgment | durability |
| REQ-MDBSRC-002.0 | TEST-INTEG-SIDEFX-004 | integration | failed send leaves records undeleted and unmarked | data safety |
| REQ-MDBSRC-003.0 | TEST-UNIT-OFFSET-005 | unit | duplicate-boundary batch cannot return `messages` with `checkpoint = None` | liveness |
| REQ-MDBSRC-003.0 | TEST-INTEG-OFFSET-006 | integration | duplicate offset group larger than batch size either drains correctly or fails fast | batch boundary |
| REQ-MDBSRC-004.0 | TEST-UNIT-TYPE-007 | unit | string `"42"` tracking offset round-trips as string BSON | type safety |
| REQ-MDBSRC-004.0 | TEST-UNIT-TYPE-008 | unit | unsupported BSON tracking type returns explicit error | validation |
| REQ-MDBSRC-005.0 | TEST-UNIT-FILTER-009 | unit | query_filter and checkpoint predicates are combined with `$and` semantics | query correctness |
| REQ-MDBSRC-005.0 | TEST-INTEG-FILTER-010 | integration | custom tracking-field filter cannot override resume position | resume correctness |
| REQ-MDBSRC-006.0 | TEST-UNIT-PAYLOAD-011 | unit | payload_field lookup is deterministic with snake_case enabled | config behavior |
| REQ-MDBSRC-007.0 | TEST-BUILD-FIXTURE-012 | build | MongoDB source fixture modules compile without sink-only dead-code warnings | scope hygiene |

## TDD Plan

### STUB

1. Add runtime-focused failing tests for source checkpoint acknowledgment.
2. Add connector unit tests for duplicate-boundary `checkpoint = None`, typed offset round-trip, and filter composition.
3. Add integration tests that inject a `producer.send()` failure or mock equivalent failure boundary.

### RED

1. Run `cargo test -p iggy_connector_mongodb_source`.
2. Run `cargo test -p integration --test mod -- mongodb_source`.
3. Record the expected failures:
   - checkpoint mutates before downstream send success
   - delete/mark runs before acknowledgment
   - numeric-string offsets are reloaded with the wrong BSON type
   - duplicate boundary with all-equal offsets has no safe progress path

### GREEN

1. Introduce a pending/acked source-state model in runtime and SDK, or block unsafe modes until that model exists.
2. Move MongoDB delete/mark work behind a post-send success path.
3. Preserve tracking offset type metadata in state, or validate and reject unsupported ambiguous tracking types.
4. Replace naive filter key insertion with explicit logical composition.
5. Remove sink-only fixture leftovers from source test support code.

### REFACTOR

1. Keep runtime/SDK changes isolated from connector-local MongoDB logic.
2. Consolidate shared poll/mark/delete filter construction behind one verified helper.
3. Prefer explicit names for pending vs acknowledged checkpoint state.

### VERIFY

1. `cargo fmt --all -- --check`
2. `cargo clippy -p iggy_connector_mongodb_source -p iggy_connector_sdk -p iggy-connectors --all-targets -- -D warnings`
3. `cargo test -p iggy_connector_mongodb_source`
4. `cargo build -p server --bin iggy-server`
5. `cargo build -p iggy-connectors --bin iggy-connectors`
6. `cargo test -p integration --test mod -- mongodb_source`
7. Confirm each `REQ-MDBSRC-*` has at least one linked test ID in this spec or code comments

## Quality Gates

- [ ] No source batch is acknowledged in connector state before `producer.send()` succeeds.
- [ ] `delete_after_read` and `processed_field` are either post-ack only or rejected as unsupported.
- [ ] Duplicate-boundary handling has an explicit safe behavior for all-equal offsets across `batch_size + 1`.
- [ ] Tracking offset state preserves BSON comparison type or rejects ambiguous types.
- [ ] `query_filter` cannot overwrite checkpoint predicates.
- [ ] MongoDB source fixture code is source-only and warning-clean.
- [ ] README delivery semantics match the actual runtime behavior.

## Open Questions

1. Should source acknowledgment be solved generically in `core/connectors/sdk` and `core/connectors/runtime`, or should MongoDB source temporarily reject all stateful modes until the runtime grows an ack hook?
2. For non-unique tracking fields, do we want a compound checkpoint `(tracking_value, _id)` or a documented “unique monotonic field required” constraint?
3. Should supported tracking types be limited to `Int64`, `String`, and `_id:ObjectId`, with explicit rejection for `DateTime`, `Decimal128`, and other BSON types until typed state is implemented?

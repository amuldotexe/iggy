# Sink-Side Sync Boundary Spec

## Objective

Define what the sink side can honestly guarantee today, what is missing in the runtime, why sink correctness is harder than source correctness, and what the smallest useful behavioral test should be.

This note treats "sync" as "sink delivery semantics from Iggy into an external system".

## Current Code Reality

### What is implemented today

1. Sources already have an explicit delivery boundary.
   - `Source::poll()` produces a batch.
   - `Source::commit_polled_messages_now()` runs only after successful downstream send.
   - `Source::discard_polled_messages_now()` runs when delivery fails.
   - See `core/connectors/sdk/src/lib.rs` and `core/connectors/runtime/src/source.rs`.

2. Sinks do not have an explicit delivery boundary.
   - `Sink` exposes only `open()`, `consume(...)`, and `close()`.
   - There is no sink-side `prepare`, `commit`, or `discard` hook.
   - See `core/connectors/sdk/src/lib.rs`.

3. Sink consumer offsets are currently advanced at poll time.
   - Runtime creates sink consumers with `AutoCommit::When(AutoCommitWhen::PollingMessages)`.
   - That means sink progress advances before the external destination acknowledges the write.
   - See `core/connectors/runtime/src/sink.rs`.

4. Existing sink integration tests are mostly happy-path storage tests.
   - PostgreSQL sink tests verify rows are written and payloads look correct.
   - Elasticsearch sink tests verify documents land and JSON shape survives.
   - These are useful, but they do not verify the sink delivery boundary.

### What is not implemented today

1. No runtime-managed pending sink batch exists.
2. No sink offset commit after external durable acknowledgment exists.
3. No sink replay contract after failure or restart exists.
4. No sink conformance kit exists for partial success, crash windows, or replay safety.
5. No capability manifest exists to distinguish:
   - at-most-once sinks
   - at-least-once sinks
   - idempotent sinks
   - append-only sinks

## First-Principles Model

For a sink, five events matter:

1. Iggy message is polled.
2. Sink tries to write externally.
3. External system acknowledges none, some, or all of the batch.
4. Iggy consumer offset is committed.
5. Process may crash before or after any of the above.

The hard problem is simple:

`done` means different things in different systems.

- Iggy thinks `done` means offset advanced.
- Destination thinks `done` means external side effect durably accepted.
- Connector code often thinks `done` means `consume(...)` returned `Ok(())`.

If those frontiers are not aligned, "sync" becomes ambiguous and bugs look like product mistakes even when the code follows the local PRD.

## Why Sink-Side Correctness Is Hard

### The true boundary

The hard boundary is not MongoDB, PostgreSQL, or Elasticsearch. The hard boundary is:

`external durable acknowledgment` vs `Iggy offset advancement`

If offset advances before durable external ack, the system is at-most-once from the destination's perspective and can lose messages on failure.

If offset advances after durable external ack, the system can be at-least-once, but only if replay is safe.

### Partial success is the sink killer

Many destinations can partially accept a batch.

Examples:

- bulk insert writes some rows and rejects some rows
- ordered insert fails after a prefix already landed
- upsert succeeds for some keys and times out for others

That creates three sink classes:

1. `Idempotent keyed sinks`
   - Example: destination supports deterministic record identity or safe upsert.
   - Retries are usually survivable.

2. `Append-only sinks with no idempotent key`
   - Retries risk duplicates.
   - Early offset commit risks loss.

3. `Partially acknowledging sinks`
   - Connector may know "something landed" but not exactly what.
   - This is the hardest class.

### Why source is easier than sink

Source correctness is easier because the source runtime already has a two-step shape:

`poll -> downstream send -> commit/discard`

Sink correctness needs the symmetric shape:

`poll -> external write -> commit/discard`

Today the sink runtime skips that symmetry and effectively behaves like:

`poll+commit -> external write`

That is the root problem.

## What A 1000 IQ Person Would Do

They would stop talking about "sync" as one thing and split it into two products:

### Product A: Honest 95% MVP

Goal:

- Works for clean happy-path usage.
- Good enough for demos, internal adoption, and most normal runs.
- Explicitly does not promise replay-safe durability.

Contract:

- Sink semantics are declared as `at_most_once`.
- Connector must not silently claim full success after partial failure.
- Connector should use deterministic external identity when possible.
- Runtime must surface failures clearly.

### Product B: Reliability-Grade Sink Delivery

Goal:

- Replay-safe sink semantics after failure or restart.

Contract:

- Runtime does not commit Iggy offsets until external durable ack.
- Sink or runtime can retry safely.
- Destination identity or sink protocol makes replay safe.

Product B is not a documentation change. It requires runtime and SDK changes.

## Implemented vs Missing

### Implemented enough for Product A

1. Happy-path end-to-end sink delivery exists.
2. Connector-local idempotency tricks can exist.
   - Example: composite document IDs, upserts, duplicate-key tolerance.
3. Backend-specific fixture tests already exist.
4. Runtime can surface sink task failures.

### Missing for Product B

1. Sink `commit/discard` lifecycle hooks
2. Runtime-held pending sink batch
3. Offset commit after external ack
4. Restart replay contract
5. Conformance tests for:
   - crash before ack
   - partial external success
   - replay after restart
   - duplicate-safe retry

## Minimal Useful Behavioral Test

There are two answers depending on which product we are trying to validate.

### Minimal test for the current 95% MVP

This is the smallest test that proves the sink is useful without pretending it is replay-safe:

`partial_external_failure_is_not_silent`

Behavior:

1. Poll a sink batch.
2. Force the destination to accept only part of the batch.
3. Make the connector return failure.
4. Assert the runtime surfaces an error.
5. Assert the destination contains only the externally acknowledged records.
6. Assert the connector does not report full success.

Why this is the right MVP test:

- It catches the most dangerous lie: "everything synced" when it did not.
- It does not require a runtime contract the system does not yet have.
- It gives operators truthful observability, which is enough for many real users.

### Minimal test for true sink conformance

This is the smallest test that proves a real sink delivery boundary:

`offset_does_not_advance_before_external_ack`

Behavior:

1. Poll a sink batch.
2. Fail the sink after polling but before durable external ack.
3. Restart the runtime.
4. Assert the same batch is replayed from Iggy.
5. Assert no message was lost.

This is the real sink conformance test.

Current runtime cannot honestly pass this because sink offsets advance during polling.

## Executable Requirements

### REQ-SINK-001.0: Honest semantics classification

**WHEN** a sink connector runs on the current runtime behavior that auto-commits offsets while polling  
**THEN** the system SHALL classify sink delivery semantics as `at_most_once`  
**AND** SHALL NOT claim `at_least_once` or `exactly_once` in docs or tests  
**SHALL** document that failure after poll may lose messages.

### REQ-SINK-002.0: Working 95% happy-path delivery

**WHEN** the destination accepts a clean batch without crash or injected failure  
**THEN** the sink SHALL persist all messages in that batch  
**AND** SHALL preserve a deterministic mapping from Iggy message identity to external records when the destination supports such identity  
**SHALL** pass an end-to-end fixture test for that behavior.

### REQ-SINK-003.0: Partial failure visibility

**WHEN** a destination partially accepts a sink batch and the connector cannot complete the full write  
**THEN** the connector SHALL surface failure explicitly  
**AND** SHALL NOT report full success for the whole batch  
**SHALL** expose exact or estimated successful write count when the backend provides enough information.

### REQ-SINK-004.0: Strong sink replay safety

**WHEN** the runtime targets `at_least_once` sink delivery  
**THEN** it SHALL NOT commit Iggy consumer offsets before durable external acknowledgment  
**AND** SHALL make the failed batch available again after restart or retry  
**SHALL** provide a sink-side commit frontier equivalent to the source-side `commit/discard` boundary.

## Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-SINK-001.0 | TEST-SINK-SEM-001 | integration | runtime and docs classify current sink mode as at-most-once | current runtime |
| REQ-SINK-002.0 | TEST-SINK-MVP-001 | integration | clean batch lands fully in destination | every sink |
| REQ-SINK-002.0 | TEST-SINK-MVP-002 | integration | deterministic external identity avoids simple multi-topic collisions | keyed sinks |
| REQ-SINK-003.0 | TEST-SINK-MVP-003 | integration | partial external failure surfaces error and does not claim full success | every sink that can partially ack |
| REQ-SINK-004.0 | TEST-SINK-ACK-001 | integration | offset does not advance before external ack | future runtime |
| REQ-SINK-004.0 | TEST-SINK-ACK-002 | restart | failed pre-ack batch replays after restart | future runtime |

## TDD Plan

### STUB

1. Write a sink-fixture failure injector that can:
   - accept all writes
   - accept a prefix then fail
   - fail before any durable write

2. Write `TEST-SINK-MVP-003` first.
   - This is the highest-signal current-runtime test.

3. Write `TEST-SINK-ACK-001` as an ignored or pending design test.
   - It should describe the desired future runtime behavior.

### RED

1. Run `TEST-SINK-MVP-003` against one sink that can partially acknowledge writes.
2. Confirm whether failure is visible and whether success is currently overstated.
3. Run `TEST-SINK-ACK-001` and confirm it fails on current runtime design.

### GREEN

1. Land connector-local fixes for honest failure visibility.
2. Do not over-claim replay safety before runtime support exists.

### REFACTOR

1. Generalize the failure injector into a sink conformance helper.
2. Add capability flags for:
   - `supports_idempotent_keys`
   - `can_partially_acknowledge`
   - `supports_safe_replay`

### VERIFY

1. Run the happy-path sink fixture tests.
2. Run the partial-failure visibility test.
3. Keep the replay-safe test as the runtime-redesign gate.

## Quality Gates

1. No sink doc may claim stronger semantics than the runtime can provide.
2. Every sink PR must declare:
   - external identity strategy
   - partial success behavior
   - claimed delivery semantics
3. Every sink must pass `TEST-SINK-MVP-001`.
4. Every partially acknowledging sink must pass `TEST-SINK-MVP-003`.
5. `TEST-SINK-ACK-001` becomes mandatory only after runtime adds sink commit/discard support.

## Open Questions

1. Should the runtime expose sink-side `commit_consumed_messages_now()` and `discard_consumed_messages_now()` to mirror sources?
2. Should sink capabilities be declared in code, config, or both?
3. Should partially acknowledging backends be required to expose exact successful write counts, or is estimated count sufficient for MVP?
4. Should the first sink conformance kit target only idempotent-key sinks and exclude append-only sinks until the runtime boundary is redesigned?

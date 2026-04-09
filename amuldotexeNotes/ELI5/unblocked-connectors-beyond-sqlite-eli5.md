# Unblocked Connectors Beyond SQLite, In Plain English

## Big Idea

If you are new and want a safe reputation win, pick connectors that do **not** force you to solve (or accidentally redefine) the connector contracts.

## Why It Matters

Right now, Apache Iggy connectors are in what we called **constitution mode**:

- the project is still deciding the shared rules for "what counts as success"
- reviews slow down when each connector reopens the same contract questions

The contract overview is captured in:

- `amuldotexeNotes/ELI5/connectors-ecosystem-contracts-eli5.md`

So the safest first connector work is the work that:

- fits the current framework shape
- is easy to validate locally
- avoids partial-write and multi-route debates

## Core Ideas Made Simple

### 1. "Unaffected" does not mean "perfect"

There are still open, unassigned sink-runtime contract issues:

- [#2927](https://github.com/apache/iggy/issues/2927) (sink `consume()` result handling)
- [#2928](https://github.com/apache/iggy/issues/2928) (sink auto-commit timing)
- [#2940](https://github.com/apache/iggy/issues/2940) (partial external writes / replay-safe progress)

That means any *new* sink connector still lives inside today’s shared sink-runtime story.

So in this note, **unaffected** means:

- it does not add new contract pressure beyond what already exists
- it is unlikely to trigger long design debates
- it stays easy to validate and review

### 2. Sources are the cleaner first lane

The current Iggy source framework is best for:

- poll-based sources
- single-topic / single-destination output

ELI5:

- it works great for a robot that checks one shelf repeatedly
- it is awkward for a robot that must sort each box into different rooms

That is why push-based / multi-route sources (like a full webhook gateway) hit framework blockers sooner.

### 3. Sinks can be safe if they are "per-message atomic"

If a sink can write each message independently (one publish, one push, one log event), it avoids the hardest partial-batch questions.

This does not eliminate the shared sink-runtime caveat.
It just keeps the connector itself simple and honest.

## Where SQLite Fits

SQLite is a good **local dev** connector (easy to demo, easy to test), and it is explicitly on the ecosystem map in [discussion #2756](https://github.com/apache/iggy/discussions/2756).

But as a sink it still inherits today’s sink-runtime semantics, so it is not truly "unaffected" in the strongest sense.

## Six "Least-Blocked" Connector Options (Beyond SQLite)

These are chosen to be serious, understandable, and low-drama given today’s framework and contract constraints.

### Sources (least blocked today)

| Connector (Source) | Two-line commentary |
|---|---|
| **S3 source** | Poll-based + single-topic fits today’s source model (no routing blockers).<br>Validate locally via MinIO; state can be “last processed key/etag”. |
| **SQS source** | Poll + delete-after-success gives a clean retry story with minimal framework drama.<br>Validate via LocalStack; keep v1 “one queue → one topic”. |
| **Redis Streams source** | Poll-based cursor maps cleanly to connector state and restart behavior.<br>Validate via Docker Redis; keep v1 “single stream → single topic”. |

### Sinks (least blocked, but still inherit sink-runtime caveat)

| Connector (Sink) | Two-line commentary |
|---|---|
| **Loki sink (logs)** | Observability sinks tolerate best-effort delivery better than “data of record” systems.<br>Keep v1 per-message or tiny batches; validate via Loki Docker. |
| **NATS sink (core publish)** | Per-message atomic publish avoids partial-batch semantics and is easy to reason about.<br>Still at-most-once under current runtime; implement retry + explicit docs. |
| **Redis sink (LPUSH / XADD)** | Per-message atomic write avoids the “half the batch succeeded” problem in [#2940](https://github.com/apache/iggy/issues/2940).<br>Same runtime caveat; validate via Docker Redis; document semantics. |

## Tiny Example (A Simple Decision Filter)

If a connector idea says:

- “one incoming message goes to different topics”
  - that is a **routing** problem, not a simple connector problem

- “I write 100 rows and might only write 60”
  - that is a **partial success** contract problem, not a simple connector problem

If a connector idea says:

- “I poll a queue and delete only after success”
  - that is a good “first source connector” shape

## What To Remember

- The fastest reputation path is: **fit the current framework**, avoid redefining the contract.
- Sources are simpler today because they fit poll-based single-destination patterns better.
- If you build a sink, pick one that is per-message atomic and be honest about semantics.
- Before you start, always re-check [discussion #2756](https://github.com/apache/iggy/discussions/2756) so you do not duplicate active work.

**Sticky sentence:** pick connectors that look boring in architecture diagrams, because boring is easier to merge.

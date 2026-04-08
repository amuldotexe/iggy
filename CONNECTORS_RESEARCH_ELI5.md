# Connectors Research, In Plain English

## Big Idea

Apache Iggy connectors are like little delivery robots: some bring data **into** Iggy, some take data **out**, and the hard part now is not "can we add more robots?" but "can we trust them when things go wrong?"

## Why It Matters

If connectors are good, Iggy becomes much easier to adopt.

People usually do not buy a message system because they love message systems.
They buy it because they want something practical like:

- Postgres data sent somewhere else
- webhooks turned into streams
- events copied into search, analytics, or storage systems

So connectors are the "bridges to real work."

As of the research done against the latest upstream discussions on April 8, 2026, the connector story is already real, not hypothetical:

- shipped connectors include PostgreSQL, Elasticsearch, Iceberg, Quickwit, Stdout, MongoDB sink, InfluxDB sink/source, and HTTP sink
- active work includes ClickHouse, Delta Lake, MongoDB source, HTTP source, S3 sink, JDBC, InfluxDB v3, and codec work like Avro and BSON

Main discussion references:

- [Connector ecosystem tracking #2756](https://github.com/apache/iggy/discussions/2756)
- [Generic HTTP sink #2919](https://github.com/apache/iggy/discussions/2919)
- [HTTP header forwarding #3029](https://github.com/apache/iggy/discussions/3029)
- [HTTP source / webhook gateway #3039](https://github.com/apache/iggy/discussions/3039)

## Core Ideas Made Simple

### 1. What a connector really is

Think of Iggy as a giant mailbox system.

- A **source connector** is a robot that picks up letters from outside and puts them into the mailbox.
- A **sink connector** is a robot that opens the mailbox and delivers letters somewhere else.
- The **runtime** is the building manager that starts the robots, gives them config, checks if they are healthy, and stops them when needed.

### 2. What is already strong

The runtime already looks much more serious than a toy plugin loader.

It already has:

- local and HTTP-based config providers
- versioned source and sink configs
- runtime HTTP APIs
- metrics and stats
- restart and lifecycle controls

You can see that in the runtime docs:

- [runtime README](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/runtime/README.md#L41)
- [runtime README](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/runtime/README.md#L147)

That means the codebase already has the bones of a connector **product**, not just a folder of adapter crates.

### 3. The hardest sink problem

This is the biggest plain-English problem:

> "Did we mark the message as done before it was actually delivered?"

Right now, the sink runtime still auto-commits progress before the sink finishes processing:

- [sink runtime](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/runtime/src/sink.rs#L418)

Also, the runtime currently calls the sink callback and throws away the return status:

- [sink runtime](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/runtime/src/sink.rs#L585)

ELI5:

- imagine a courier saying "package delivered" before reaching the house
- then the van crashes
- now the system thinks the job is finished, but the package never arrived

That is why the hardest connectors problem is really a **delivery truth** problem.

### 4. The hardest source problem

This is the plain-English version:

> "Can one source connector send different messages to different destinations?"

The current source message shape has no destination field:

- [connector SDK](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/sdk/src/lib.rs#L301)

So advanced push-style sources like an HTTP webhook gateway run into a wall:

- the connector may know exactly which topic each incoming request should go to
- but the SDK cannot express that routing cleanly yet

That is why the HTTP source discussion became important. It exposed a real contract gap, not just a missing connector:

- [HTTP source discussion #3039](https://github.com/apache/iggy/discussions/3039)

### 5. The shutdown problem

Another simple way to say it:

> "When we stop the robot, do we let it finish the box in its hands?"

On the source side, cleanup currently happens before `iggy_source_close()`:

- [source manager](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/runtime/src/manager/source.rs#L151)

That means a source can still be trying to send messages during shutdown, but the path it needs has already been removed.

ELI5:

- the building manager closes the loading dock
- then tells the robot to drop off its final boxes
- the robot now has nowhere to place them

### 6. Why the Pareto test suites matter

The Pareto source/sink test issues are not just "we need more tests."

They are really asking four grown-up questions in a very healthy way:

1. How do we know what has already been read or written?
2. When do we call a message successfully handled?
3. What can be replayed, duplicated, or lost after failure?
4. What is the smallest test that proves this?

That is the right shape of thinking for connector quality.

### 7. Why HTTP became a big deal

The shipped HTTP sink is important because it showed two things at once:

- a generic connector can unlock lots of integrations quickly
- shared runtime problems immediately become visible when a connector is used seriously

The HTTP discussions also suggest a future shared HTTP utility layer:

- one reusable client
- one retry engine
- one metadata envelope story
- one place for observability and conventions

That would be much cleaner than every HTTP-ish sink writing its own mini-framework.

## Tiny Example

Here is the simple mental model for the hardest sink bug:

```text
Iggy message arrives
    |
    v
runtime says "I will remember this offset"
    |
    v
sink tries to write to external system
    |
    +--> success: good
    |
    +--> crash/failure: bad, because progress may already look committed
```

The fix direction is also simple in plain English:

```text
do the work first
only mark progress after success
make failures visible
test restart and replay behavior
```

## What To Remember

- Connector breadth matters, but connector **truthfulness** matters more.
- The runtime already has strong product-like pieces: config APIs, stats, metrics, and lifecycle controls.
- The hardest unsolved problems are shared runtime contracts:
  - sink commit timing
  - callback error handling
  - source routing shape
  - shutdown ordering
  - replay/duplicate/loss behavior
- The healthiest next work is not random connector expansion.
  It is:
  - fixing shared runtime semantics
  - documenting behavior contracts
  - adding Pareto-style tests
  - then building more connectors on top of firmer ground

Good connectors are not just bridges. They are bridges that still tell the truth when the road shakes.

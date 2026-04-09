# Three Straightforward Connector PRs

## Big Idea

If we want three useful Apache Iggy connector PRs that are easier to own than a brand-new connector, the best bets are: Pinot end-to-end tests, a small shared source behavior test suite, and a small shared sink behavior test suite.

## Why It Matters

Think of the connector ecosystem like a growing workshop.

- New connectors are new machines.
- Tests are the safety rails and measuring tools.
- Behavior contracts are the labels that say when a machine is actually done.

Right now, some of the highest-value work is not "build a whole new machine." It is "make the workshop easier to trust."

This note is based on a GitHub CLI survey and focused issue traces collected on April 9, 2026. The recommendations below are a mix of:

- facts from GitHub issues, PRs, and discussions
- facts from the local Iggy codebase
- cautious interpretation about which lanes look low-risk and still open

## Core Ideas Made Simple

### 1. Apache Pinot end-to-end tests are the cleanest finish-the-last-mile PR

Issue: [#2598](https://github.com/apache/iggy/issues/2598)

Plain-English version:

The Pinot connector already exists, but its real integration test still lives as a shell script. That is like having a working recipe written on a napkin instead of in the cookbook the rest of the team uses.

Why this is attractive:

- The issue is already shaped as test work, not architecture work.
- The Java module already has test dependencies wired in.
- The manual test flow already exists and can be translated into proper Java/Testcontainers coverage.
- The roadmap discussion still lists this as open and useful.

What the repo already gives us:

- The Pinot module already has Gradle test wiring in `foreign/java/external-processors/iggy-connector-pinot/build.gradle.kts`.
- The current manual flow is spelled out in `foreign/java/external-processors/iggy-connector-pinot/integration-test.sh`.
- The issue body already defines acceptance criteria: 2-3 Java tests, Testcontainers, Gradle CI integration, and removal of the old shell flow.

Important nuance:

- The issue is currently assigned, so this is a "comment first" lane, not a "surprise PR" lane.
- The latest discussion suggests the blocker is mostly execution friction and version drift, not deep design uncertainty.

### 2. Pareto source connector test suite is the best leverage PR

Issue: [#2892](https://github.com/apache/iggy/issues/2892)

Plain-English version:

This is like writing one good driving test that every new source connector can take, instead of inventing a new driving test for each car.

Why this is attractive:

- It improves every future source connector, not just one.
- The source runtime already has a pretty clean contract.
- Existing tests already show the pattern for restart and state checks.
- The connector roadmap and discussion both keep pointing back to the same source behavior questions.

What the repo already gives us:

- The source contract is very small: `open()`, `poll()`, `close()` in `core/connectors/sources/README.md`.
- The runtime sends messages to Iggy first and only then persists source state in `core/connectors/runtime/src/source.rs`.
- There is already a working restart/state example in `core/integration/tests/connectors/random/random_source.rs`.

Why it stays low risk:

- This is mostly additive test and harness work.
- It does not need a new connector crate.
- It does not need a runtime redesign to be useful.

Good first scope:

- verify that progress does not advance before send succeeds
- verify restart resumes from committed state
- wire one existing source into the shared suite
- document how another source would plug in later

### 3. Pareto sink connector test suite is also high value, but slightly less settled

Issue: [#2893](https://github.com/apache/iggy/issues/2893)

Plain-English version:

This is the sink-side version of the same idea: one common checklist for "did this connector really write data safely?" instead of repeating the same lessons in every sink PR.

Why this is attractive:

- It raises the quality floor across all sink connectors.
- It captures tricky replay and failure behavior in one place.
- The GitHub discussion shows maintainers and contributors keep returning to these same questions.

What makes it a little less clean than the source suite:

- Sink behavior still has open runtime questions.
- The runtime currently auto-commits offsets before sink processing in `core/connectors/runtime/src/sink.rs`.
- The HTTP sink tests already document this limitation in `core/integration/tests/connectors/http/http_sink.rs`.

That means the safest first version is:

- test and document current behavior clearly
- make duplicate and loss windows explicit
- avoid pretending the suite solves the open runtime design questions by itself

In other words:

This is still a good PR, but it should be framed as "shared behavior coverage for today" rather than "final sink semantics forever."

## Why These Three Beat Other Open Connector Ideas

Some open items look important, but they are not clean newcomer lanes right now.

### Why not the sink runtime `consume()` fix?

Issue: [#2927](https://github.com/apache/iggy/issues/2927)

Because it already has an open PR: [#3061](https://github.com/apache/iggy/pull/3061).

That means the lane is not really open anymore. It may still need review help, but it is not a fresh, straightforward PR to own from scratch.

### Why not the auto-commit sink runtime bug?

Issue: [#2928](https://github.com/apache/iggy/issues/2928)

Because it is important, but not low-risk. It touches delivery guarantees and replay semantics for all sinks. That is more like changing the rules of the road than repainting lane markers.

### Why not a new big connector like ClickHouse, JDBC, or S3 sink?

Because those are larger product lanes with more design surface, and some already have active owners or PRs.

- ClickHouse sink already has an open PR: [#2886](https://github.com/apache/iggy/pull/2886)
- Iceberg default credential chain already has an open PR: [#3045](https://github.com/apache/iggy/pull/3045)
- S3 sink is promising, but it is still a real connector implementation, not a low-risk cleanup or coverage PR

## Tiny Example

Here is the simple difference between a "new connector" PR and a "shared behavior suite" PR:

Bad framing:

`Let's build one more connector and hope we remember the tricky restart rules later.`

Better framing:

`Let's write one shared test that checks restart behavior once, then reuse it for every future connector.`

Why the second is better:

- it reduces repeated mistakes
- it lowers review effort
- it helps newcomers know what "done" means

## Recommended Order

If we want to sequence the work by ease and value, the order should be:

1. Pinot e2e tests if the current assignee is okay with help
2. Pareto source connector test suite
3. Pareto sink connector test suite

If the Pinot lane is still actively owned, then the two Pareto suites become the cleanest open lanes.

## What To Remember

The easiest valuable connector PR is often not "add one more connector," but "make every connector easier to trust."

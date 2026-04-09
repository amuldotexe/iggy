# Non-Connector Open Issues: Digest

Generated from `/tmp/iggy-current-open-issues.json` on 2026-04-09T09:12:39Z (UTC).

## #2396: [Nodejs SDK] should Client expose  event handler for events through on/once
- URL: https://github.com/apache/iggy/issues/2396
- Labels: good first issue, javascript
- Updated: 2026-04-05T08:10:37Z
- Comments captured: 2
- Summary: To simplify and standardize how events are registered, should we add on/once methods to the handler event?

## #3075: Address validation for `QuicClientConfigBuilder` and `HttpClientConfigBuilder`
- URL: https://github.com/apache/iggy/issues/3075
- Updated: 2026-04-04T20:39:22Z
- Comments captured: 0
- PR hint: yes
- Summary: ### Description

## #3030: (cli): `context use` can fail on fresh system when `~/.iggy/` directory does not exist
- URL: https://github.com/apache/iggy/issues/3030
- Updated: 2026-04-02T17:16:49Z
- Comments captured: 2
- PR hint: yes
- Summary: The `context use` command writes to `~/.iggy/.active_context` via `tokio::fs::write`, which fails if the `~/.iggy/` directory hasn't been created yet.

## #2985: auto generate helm readme so that drift between document and code remains at minimum + beautify(fix formatting issues) YAML
- URL: https://github.com/apache/iggy/issues/2985
- Assignees: avirajkhare00
- Updated: 2026-04-02T00:17:45Z
- Comments captured: 5
- Summary: Context

## #2872: Add integration test for message deduplication
- URL: https://github.com/apache/iggy/issues/2872
- Labels: good first issue, server, test
- Assignees: seokjin0414
- Updated: 2026-04-01T19:25:25Z
- Comments captured: 3
- Summary: Message deduplication has unit tests in `MessageDeduplicator` but no integration test verifying the full pipeline: client sends duplicate messages -> server drops them at partition level.
- Paths: core/common/src/deduplication/message_deduplicator.rs, core/common/src/types/message/messages_batch_mut.rs, core/server/src/streaming/partitions/helpers.rs

## #1420: Implement Direct I/O, bypass kernel page cache
- URL: https://github.com/apache/iggy/issues/1420
- Labels: io_uring, performance, tpc
- Assignees: tungtose
- Updated: 2026-03-30T17:28:55Z
- Comments captured: 3

## #2699: Normalize enum serialization strategy
- URL: https://github.com/apache/iggy/issues/2699
- Updated: 2026-03-30T07:53:14Z
- Comments captured: 1
- PR hint: yes
- Summary: Some enums are missing the `serde` rename macro (e.g. `SystemSnapshotType`, `SnapshotCompression`), while others use`#[serde(rename_all = "lowercase")]` instead of snake_case.

## #3000: (cli): Add `context show` and `session status` commands to CLI
- URL: https://github.com/apache/iggy/issues/3000
- Updated: 2026-03-25T19:06:39Z
- Comments captured: 1
- Summary: The context and login session workflow is missing two observability commands that would improve the developer experience.

## #19: Implement & expose custom server metrics for Prometheus
- URL: https://github.com/apache/iggy/issues/19
- Labels: api, good first issue
- Updated: 2026-03-25T16:01:53Z
- Comments captured: 5

## #2965: [C++ SDK] implement `bdd/scenarios/basic_messaging.feature` test for C++ SDK
- URL: https://github.com/apache/iggy/issues/2965
- Labels: C++, CI/CD, good first issue, test
- Assignees: seokjin0414
- Updated: 2026-03-23T13:20:36Z
- Comments captured: 3
- Summary: Add BDD-style integration test for the C++ SDK using a Gherkin-compatible framework (e.g. [cucumber-cpp](https://github.com/cucumber/cucumber-cpp) or [Catch2 BDD](https://github.com/catchorg/Catch2/blob/devel/docs/test-cases-and-sections.md#bdd-style-test-cas…
- Paths: scripts/run-bdd-tests.sh

## #3012: NixOS packages and Module.
- URL: https://github.com/apache/iggy/issues/3012
- Assignees: MathisWellmann
- Updated: 2026-03-23T10:02:47Z
- Comments captured: 1
- Summary: ### Description

## #2628: [csharp SDK] Implement leader_redirection scenario in BDD tests
- URL: https://github.com/apache/iggy/issues/2628
- Labels: csharp, good first issue
- Assignees: yeyomontana
- Updated: 2026-03-23T09:05:06Z
- Comments captured: 2
- Summary: Add the missing `leader_redirection` (located in `bdd/scenarios/basic_messaging.feature`) BDD test scenario to the C# SDK test suite.

## #469: Write fuzzing testcases for `iggy-server`
- URL: https://github.com/apache/iggy/issues/469
- Labels: good first issue, server
- Assignees: krishvishal
- Updated: 2026-03-23T04:34:50Z
- Comments captured: 3
- Summary: The aim of this issue is to create fuzz testcases that would bombard server with random data and check behavior. Server shouldn't ever crash. Assume auth is done.

## #2984: Provide a Cross-Language Conformance Testing for serizalize/deserialize logic of SDK
- URL: https://github.com/apache/iggy/issues/2984
- Labels: postponed
- Updated: 2026-03-20T10:53:28Z
- Comments captured: 1
- Summary: ### Description

## #2981: Go SDK: CreateUser omits mandatory permissions_len field when permissions are nil
- URL: https://github.com/apache/iggy/issues/2981
- Updated: 2026-03-19T09:23:07Z
- Comments captured: 1
- Summary: The Rust wire format for `CreateUser` always includes `permissions_len:u32_le` on the wire, even when `has_permissions=0`. The server-side decoder unconditionally reads these 4 bytes (see [create_user.rs#L82-L83](https://github.com/apache/iggy/blob/master/cor…
- Paths: core/binary_protocol/src/requests/users/create_user.rs, foreign/go/internal/command/user.go

## #2982: Go SDK: UpdatePermissions omits mandatory `permissions_len` field when permissions are nil
- URL: https://github.com/apache/iggy/issues/2982
- Assignees: atharvalade
- Updated: 2026-03-19T09:20:48Z
- Comments captured: 1
- Summary: Same root cause as the `CreateUser` issue. The Rust wire format for `UpdatePermissions` always includes `permissions_len:u32_le` on the wire regardless of `has_permissions` (see [update_permissions.rs#L37-L39](https://github.com/apache/iggy/blob/master/core/b…
- Paths: core/binary_protocol/src/requests/users/update_permissions.rs, foreign/go/internal/command/user.go

## #2978: Increase Go Codecov Coverage
- URL: https://github.com/apache/iggy/issues/2978
- Assignees: atharvalade
- Updated: 2026-03-19T08:39:39Z
- Comments captured: 1
- Summary: Go SDK Codecov coverage is only [36.37%.](https://app.codecov.io/github/apache/iggy/flags?historicalTrend=LAST_7_DAYS). We need to increase this number.

## #2715: Non-deterministic consumer offset jump to "latest" on large streams (~50M records)
- URL: https://github.com/apache/iggy/issues/2715
- Labels: bug
- Updated: 2026-03-18T15:59:10Z
- Comments captured: 21
- Summary: I am encountering a critical issue where a consumer unexpectedly jumps from an intermediate offset directly to the end of the stream (the latest offset). This behavior is non-deterministic and occurs after ingesting a large volume of data (~50M records).

## #2148: Add more BDD test scenarios.
- URL: https://github.com/apache/iggy/issues/2148
- Labels: good first issue
- Updated: 2026-03-18T10:18:04Z
- Comments captured: 1
- Summary: Currently, we only have a single feature file `basic_messaging.feature` under `bdd/scenarios`, which is clearly insufficient for ensuring full coverage of the system. We should add more BDD test scenarios to cover a broader range of functionality for SDKs. It…

## #2146: Add Migration Guides to Docs
- URL: https://github.com/apache/iggy/issues/2146
- Labels: docs, good first issue
- Assignees: atharvalade
- Updated: 2026-03-18T10:14:40Z
- Comments captured: 2
- Summary: It would be super helpful to write sample migration guides for folks coming from Kafka, Redpanda etc. We can pick 1 or 2 languages (say Java, Python) to illustrate the Client code for Publishing (writing) and Subscribing (reading) the messages

## #477: Write wireshark dissector for `iggy` protocol
- URL: https://github.com/apache/iggy/issues/477
- Labels: enhancement, good first issue, postponed
- Assignees: YangSiJun528
- Updated: 2026-03-18T10:10:01Z
- Comments captured: 2
- Summary: As in title. Protocol specs are somewhere in the docs.

## #2776: [Python SDK] Create `sync-python-version.sh` to keep python versions in sync across folders and docker images
- URL: https://github.com/apache/iggy/issues/2776
- Labels: CI/CD, good first issue, python
- Updated: 2026-03-16T01:20:09Z
- Comments captured: 2
- Summary: ### Description
- Paths: .github/workflows/_build_python_wheels.yml, .github/workflows/_common.yml, scripts/ci/sync-python-version.sh

## #2883: Go SDK: Add unit tests for command serialization/deserialization
- URL: https://github.com/apache/iggy/issues/2883
- Assignees: saie-ch
- Updated: 2026-03-11T20:57:48Z
- Comments captured: 1
- Summary: ## Description The Go SDK contains numerous command types that implement the `MarshalBinary` interface to handle binary serialization. Currently, many of these implementations lack comprehensive unit tests, which is essential for ensuring data integrity and p…

## #1594: Explore using arrayvec/arraystring crate for stack allocated strings
- URL: https://github.com/apache/iggy/issues/1594
- Labels: enhancement, performance, postponed, rust, sdk, server
- Updated: 2026-03-09T19:05:01Z
- Comments captured: 4
- Summary: Since our strings (for identifier etc..) are fixed size, we could easily move the allocation from heap to the stack. Rust doesn't have std lib support for such things, but fortunately a crate that solves that problem [https://crates.io/crates/arrayvec](url)

## #88: Add feature flag for conditional compilation of protocols (TCP, QUIC, HTTP)
- URL: https://github.com/apache/iggy/issues/88
- Labels: good first issue, postponed
- Assignees: seokjin0414
- Updated: 2026-03-05T11:57:58Z
- Comments captured: 9
- Summary: So the out-binary (or library) can be smaller.
- Paths: core/server/src/binary/sender.rs

## #2835: feat(python): Add QUIC, HTTP, and WebSocket Transport Protocol Support to Python SDK
- URL: https://github.com/apache/iggy/issues/2835
- Assignees: saie-ch
- Updated: 2026-03-02T08:04:43Z
- Comments captured: 1
- Summary: ### Description
- Paths: core/common/src/types/configuration/auth_config/connection_string.rs, core/common/src/types/configuration/transport.rs, core/integration/tests/server/cg.rs

## #2830: [C++ SDK] Consider converting high-level classes in iggy.hpp to rust-based enums
- URL: https://github.com/apache/iggy/issues/2830
- Updated: 2026-02-27T13:06:30Z
- Comments captured: 0
- Summary: ### Description

## #2827: Replace `DashMap` with `papaya` for `shards_table` in server shard
- URL: https://github.com/apache/iggy/issues/2827
- Assignees: krishvishal
- Updated: 2026-02-27T07:16:39Z
- Comments captured: 1
- Summary: The shards_table (`DashMap<IggyNamespace, PartitionLocation>`) is the shared lookup table that maps partition namespaces to their owning shard. It is queried on every message routing decision (find_shard, resolve) but only written to during partition creation…

## #2100: Implement C++ SDK
- URL: https://github.com/apache/iggy/issues/2100
- Labels: C++, sdk
- Assignees: slbotbm
- Updated: 2026-02-20T07:08:10Z
- Comments captured: 7
- Summary: Currently, we have non-working skeleton of C++ SDK in `foreign/cpp` directory. The aim of this task is to write minimal C++ SDK which will allow users to send and poll.

## #2763: [C++ SDK] Create Rust side FFI bindings for the C++ SDK
- URL: https://github.com/apache/iggy/issues/2763
- Labels: C++, sdk
- Assignees: slbotbm
- Updated: 2026-02-20T07:07:39Z
- Comments captured: 0
- Summary: ### Description

## #2764: [C++ SDK] Create high level client from the generated bindings.
- URL: https://github.com/apache/iggy/issues/2764
- Labels: C++, sdk
- Assignees: slbotbm
- Updated: 2026-02-20T07:07:34Z
- Comments captured: 0
- Summary: ### Description

## #2205: [Java SDK] Add comprehensive offset management for consumer groups
- URL: https://github.com/apache/iggy/issues/2205
- Labels: enhancement, java
- Assignees: ex172000
- Updated: 2026-02-11T17:59:23Z
- Comments captured: 1
- PR hint: yes
- Summary: **Title:** `[Java SDK] Add comprehensive offset management for consumer groups`

## #18: Server-side message compression feature
- URL: https://github.com/apache/iggy/issues/18
- Labels: config, performance
- Assignees: numinnex
- Updated: 2026-02-10T22:24:36Z
- Comments captured: 0

## #21: Configurable message size threshold
- URL: https://github.com/apache/iggy/issues/21
- Labels: config, good first issue
- Assignees: BartoszCiesla
- Updated: 2026-02-10T22:24:35Z
- Comments captured: 3

## #1714: Implement synchronous SDK client
- URL: https://github.com/apache/iggy/issues/1714
- Labels: sdk
- Updated: 2026-02-10T22:24:31Z
- Comments captured: 3
- PR hint: yes
- Summary: There might be specific use cases, when making use of the synchronous client yields a better performance than with the async one. For example, when building the ultra-low latency system, with threads pinned to the cores - the async client is pretty much usele…
- Paths: examples/server.rs

## #200: PGO in CI
- URL: https://github.com/apache/iggy/issues/200
- Labels: server
- Updated: 2026-02-10T21:08:46Z
- Comments captured: 0
- Summary: Aim of this task is to use profile guided optimization in CI.

## #2687: Implement partitions replicated log.
- URL: https://github.com/apache/iggy/issues/2687
- Labels: cluster
- Assignees: numinnex
- Updated: 2026-02-05T17:26:34Z
- Comments captured: 0
- Summary: Implement segmented log that is replicated for `partitions` module.

## #2228: [Java SDK] Add Performance Benchmarks for Java SDK
- URL: https://github.com/apache/iggy/issues/2228
- Labels: java, test
- Updated: 2026-02-05T02:37:44Z
- Comments captured: 5
- Summary: ### Title Implement JMH performance benchmarks for blocking and async clients
- Tasks: Set up JMH (Java Microbenchmark Harness) in the project | Create benchmarks for message operations: | Single message send latency

## #2203: [Java SDK] Add batch send/receive operations for better throughput
- URL: https://github.com/apache/iggy/issues/2203
- Labels: enhancement, java
- Assignees: mmodzelewski
- Updated: 2026-01-28T11:33:48Z
- Comments captured: 0
- Summary: **Title:** `[Java SDK] Add batch send/receive operations for better throughput`

## #2210: [Java SDK] Add AutoCloseable support and proper cleanup
- URL: https://github.com/apache/iggy/issues/2210
- Labels: enhancement, java
- Assignees: mmodzelewski
- Updated: 2026-01-28T11:33:23Z
- Comments captured: 0
- Summary: **Title:** `[Java SDK] Add AutoCloseable support and proper cleanup`

## #2206: [Java SDK] Implement pluggable serialization framework
- URL: https://github.com/apache/iggy/issues/2206
- Labels: enhancement, java
- Assignees: mmodzelewski
- Updated: 2026-01-28T11:33:13Z
- Comments captured: 0
- Summary: **Title:** `[Java SDK] Implement pluggable serialization framework`

## #2562: Clarify Distributed Clustering Status and Non-Linux Platform Support
- URL: https://github.com/apache/iggy/issues/2562
- Assignees: Jai-76
- Updated: 2026-01-22T04:31:41Z
- Comments captured: 5
- Summary: : Problem: The documentation, particularly the Architecture and Introduction sections, highlights Iggy as a high-performance alternative to Kafka. However, it currently lacks explicit information regarding two critical deployment factors:

## #2590: Remove overreliance on Identifier from server
- URL: https://github.com/apache/iggy/issues/2590
- Labels: cluster, metadata, server
- Updated: 2026-01-20T15:51:37Z
- Comments captured: 1
- PR hint: yes
- Summary: We have an Identifier struct, that we use to store both numeric as well as string variant of id's. SDK user can use the Identifier to name resources such as streams/topics/consumer_groups.

## #2524: Figure out how to extract `IggyError` into an dedicated crate.
- URL: https://github.com/apache/iggy/issues/2524
- Updated: 2025-12-30T14:08:15Z
- Comments captured: 2
- Summary: #2519 prompted us with a good idea, we can use our rust error as a base for code generators in other SDKs to generate error status codes. The tricky bit is to figure out project structure that would allow us to decouple the `IggyError` from `server` crate and…

## #2517: End-to-end test suite for Apache Iggy Web UI
- URL: https://github.com/apache/iggy/issues/2517
- Labels: test, web
- Assignees: rustworthy
- Updated: 2025-12-26T20:45:23Z
- Comments captured: 5
- Summary: As per [this Discord discussion](https://discordapp.com/channels/1144142576266530928/1144143410480038009/1454032941792493653) opening an issue to track work on the end-to-end tests for Web UI.
- Tasks: e2e tests setup (dependencies and utilities); | test authentication (I think I might combine it with the first step, but let's see down the road) | test main scenarios/pages (we can outline these later on)

## #2386: Transfer TCP connections across shards (only for `PollMessages`/`SendMessages` commands)
- URL: https://github.com/apache/iggy/issues/2386
- Labels: performance, server
- Assignees: tungtose
- Updated: 2025-12-09T06:49:03Z
- Comments captured: 5
- Summary: In Iggy's shared-nothing architecture, each shard owns specific partitions based on consistent hashing. When a client connected to Shard A tries to access a partition owned by Shard B, Shard A creates message to Shard B, sends it, waits for reply and sends re…

## #46: Add handling of maximum size of `logs` folder
- URL: https://github.com/apache/iggy/issues/46
- Labels: enhancement, good first issue
- Assignees: Svecco
- Updated: 2025-12-07T01:46:05Z
- Comments captured: 4
- Summary: Perhaps it should be possible to use crate https://docs.rs/rolling-file/latest/rolling_file/ instead of `tracing-appender`.
- Paths: core/server/src/log/logger.rs

## #1914: Implement Clustering (VSR)
- URL: https://github.com/apache/iggy/issues/1914
- Updated: 2025-11-30T16:43:14Z
- Comments captured: 3
- Summary: ### This issue provides insight into the technical details of implementing View-stamped Replication for Apache Iggy.
- Tasks: Clocking mechanism | Quorum | Internode Wire Protocol

## #2373: Implement generic Message Header system for Consensus
- URL: https://github.com/apache/iggy/issues/2373
- Assignees: krishvishal
- Updated: 2025-11-25T16:44:04Z
- Comments captured: 7
- Summary: We need to implement a type safe, zero copy message header system for the protocol.
- Paths: core/common/src/types/consensus/mod.rs, core/server/src/binary/command.rs

## #1890: Add Node.js SDK examples
- URL: https://github.com/apache/iggy/issues/1890
- Labels: good first issue
- Assignees: dajneem23
- Updated: 2025-11-21T16:28:26Z
- Comments captured: 5
- PR hint: yes
- Summary: Following the reorganization of our examples directory structure (with Rust examples now in `examples/rust/`), we need to create comprehensive Node.js examples to showcase the Iggy Node.js SDK capabilities.
- Paths: scripts/run-rust-examples-from-readme.sh

## #1969: Add official Ruby sdk
- URL: https://github.com/apache/iggy/issues/1969
- Updated: 2025-11-18T12:22:56Z
- Comments captured: 5
- Summary: I'm super thrilled about Iggy! 🎉

## #2207: Schema registry for data governance
- URL: https://github.com/apache/iggy/issues/2207
- Updated: 2025-10-20T14:48:38Z
- Comments captured: 2
- Summary: **Title:** `Schema registry for data governance`

## #2209: [Java SDK] Implement metrics collection and monitoring
- URL: https://github.com/apache/iggy/issues/2209
- Labels: enhancement, java
- Updated: 2025-10-06T21:00:21Z
- Comments captured: 0
- Summary: **Title:** `[Java SDK] Implement metrics collection and monitoring`

## #2226: [Java SDK] Add Error Handling and Edge Case Tests for Async Client
- URL: https://github.com/apache/iggy/issues/2226
- Labels: enhancement, java
- Updated: 2025-10-06T20:57:49Z
- Comments captured: 0
- Summary: ### Title Improve error handling test coverage for async TCP client
- Tasks: Add connection failure tests: | Server unreachable | Invalid host/port

## #2229: [Java SDK] Implement Load Testing Suite
- URL: https://github.com/apache/iggy/issues/2229
- Labels: java, test
- Updated: 2025-10-06T20:57:06Z
- Comments captured: 0
- Summary: ### Title Create load testing suite for stress testing Iggy Java clients
- Tasks: Create load test framework: | Configurable message rates | Configurable client counts

## #1986: Rewrite golang BDD tests to use scenarios
- URL: https://github.com/apache/iggy/issues/1986
- Updated: 2025-07-09T14:19:29Z
- Comments captured: 0
- Summary: I noticed that the Golang BDD tests do not use the scenarios from [bdd/scenarios/](https://github.com/apache/iggy/tree/master/bdd/scenarios). This should be changed. As stated in [Readme.md](https://github.com/apache/iggy/blob/master/bdd/README.md), all SDKs…
- Paths: bdd/README.md

## #1710: [ feature request ] Feature to replace dependency on libdbus-sys via keyring
- URL: https://github.com/apache/iggy/issues/1710
- Updated: 2025-04-18T17:15:34Z
- Comments captured: 0
- Summary: See https://github.com/diwic/dbus-rs/issues/497

## #1419: Implement Tiered-Storage
- URL: https://github.com/apache/iggy/issues/1419
- Updated: 2025-01-21T12:39:51Z
- Comments captured: 6
- Summary: Iggy should support configurable Tiered-storage functionality, to flush the data to long term storage like S3/GCS/ObjectStorage

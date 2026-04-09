# Non-Connector Open Issues: Solution Map

Generated from `/tmp/iggy-current-open-issues.json` on 2026-04-09T09:13:44Z (UTC).

## Scoring (Heuristic)
- Simplicity score: -3..6 (higher means more explicit + smaller)
- Value score: 0..8 (higher means more user impact)
- These are guides, not truth; the issue body is the source of truth.

## Likely Simple + Valuable (Unassigned)
- (none found by heuristic; see full table and consider medium-complexity items)

## Full Table (All Non-Connector Open Issues)

| issue | surface | ownership | simp | value | shape | path hints |
| --- | --- | --- | ---: | ---: | --- | --- |
| [#18](https://github.com/apache/iggy/issues/18) Server-side message compression feature | core/other | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#19](https://github.com/apache/iggy/issues/19) Implement & expose custom server metrics for Prometheus | core/other | claimed | -3 | 5 | Write plan/RFC, split into sub-issues, then implement. |  |
| [#21](https://github.com/apache/iggy/issues/21) Configurable message size threshold | core/other | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#46](https://github.com/apache/iggy/issues/46) Add handling of maximum size of `logs` folder | core/server | assigned | 1 | 0 | Investigate surface, propose small first PR. | core/server/src/log/logger.rs |
| [#88](https://github.com/apache/iggy/issues/88) Add feature flag for conditional compilation of protocols (TCP, QUIC, HTTP) | core/server | assigned | -2 | 0 | Confirm scope/timing; likely defer. | core/server/src/binary/sender.rs |
| [#200](https://github.com/apache/iggy/issues/200) PGO in CI | core/server | unassigned | 0 | 0 | Add script/workflow check + CI gating. |  |
| [#469](https://github.com/apache/iggy/issues/469) Write fuzzing testcases for `iggy-server` | core/server | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#477](https://github.com/apache/iggy/issues/477) Write wireshark dissector for `iggy` protocol | core/other | assigned | -3 | 0 | Confirm scope/timing; likely defer. |  |
| [#1419](https://github.com/apache/iggy/issues/1419) Implement Tiered-Storage | core/other | unassigned | -3 | 2 | Write plan/RFC, split into sub-issues, then implement. |  |
| [#1420](https://github.com/apache/iggy/issues/1420) Implement Direct I/O, bypass kernel page cache | core/other | assigned | -3 | 5 | Write plan/RFC, split into sub-issues, then implement. |  |
| [#1594](https://github.com/apache/iggy/issues/1594) Explore using arrayvec/arraystring crate for stack allocated strings | core/server | unassigned | -3 | 0 | Confirm scope/timing; likely defer. |  |
| [#1710](https://github.com/apache/iggy/issues/1710) [ feature request ] Feature to replace dependency on libdbus-sys via keyring | core/other | unassigned | 0 | 0 | Investigate surface, propose small first PR. |  |
| [#1714](https://github.com/apache/iggy/issues/1714) Implement synchronous SDK client | core/other | unassigned | 0 | 2 | Write plan/RFC, split into sub-issues, then implement. | examples/server.rs |
| [#1890](https://github.com/apache/iggy/issues/1890) Add Node.js SDK examples | CI/CD | assigned | 1 | 0 | Investigate surface, propose small first PR. | scripts/run-rust-examples-from-readme.sh |
| [#1914](https://github.com/apache/iggy/issues/1914) Implement Clustering (VSR) | core/other | unassigned | -3 | 2 | Write plan/RFC, split into sub-issues, then implement. |  |
| [#1969](https://github.com/apache/iggy/issues/1969) Add official Ruby sdk | core/other | unassigned | 0 | 0 | Add script/workflow check + CI gating. |  |
| [#1986](https://github.com/apache/iggy/issues/1986) Rewrite golang BDD tests to use scenarios | core/other | unassigned | 2 | 2 | Add one scenario/test + wire into harness. | bdd/README.md |
| [#2100](https://github.com/apache/iggy/issues/2100) Implement C++ SDK | foreign/cpp + Rust FFI | assigned | -3 | 2 | Write plan/RFC, split into sub-issues, then implement. |  |
| [#2146](https://github.com/apache/iggy/issues/2146) Add Migration Guides to Docs | docs | assigned | -1 | 1 | Write/extend docs; add examples. |  |
| [#2148](https://github.com/apache/iggy/issues/2148) Add more BDD test scenarios. | core/other | unassigned | 0 | 0 | Add one scenario/test + wire into harness. |  |
| [#2203](https://github.com/apache/iggy/issues/2203) [Java SDK] Add batch send/receive operations for better throughput | foreign/java | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2205](https://github.com/apache/iggy/issues/2205) [Java SDK] Add comprehensive offset management for consumer groups | foreign/java | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2206](https://github.com/apache/iggy/issues/2206) [Java SDK] Implement pluggable serialization framework | foreign/java | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2207](https://github.com/apache/iggy/issues/2207) Schema registry for data governance | core/other | unassigned | -2 | 0 | Investigate surface, propose small first PR. |  |
| [#2209](https://github.com/apache/iggy/issues/2209) [Java SDK] Implement metrics collection and monitoring | foreign/java | unassigned | 0 | 3 | Investigate surface, propose small first PR. |  |
| [#2210](https://github.com/apache/iggy/issues/2210) [Java SDK] Add AutoCloseable support and proper cleanup | foreign/java | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2226](https://github.com/apache/iggy/issues/2226) [Java SDK] Add Error Handling and Edge Case Tests for Async Client | foreign/java | unassigned | 0 | 2 | Add one scenario/test + wire into harness. |  |
| [#2228](https://github.com/apache/iggy/issues/2228) [Java SDK] Add Performance Benchmarks for Java SDK | foreign/java | claimed | -1 | 5 | Add one scenario/test + wire into harness. |  |
| [#2229](https://github.com/apache/iggy/issues/2229) [Java SDK] Implement Load Testing Suite | foreign/java | unassigned | 0 | 2 | Add one scenario/test + wire into harness. |  |
| [#2373](https://github.com/apache/iggy/issues/2373) Implement generic Message Header system for Consensus | core/server | assigned | -1 | 2 | Write plan/RFC, split into sub-issues, then implement. | core/common/src/types/consensus/mod.rs, core/server/src/binary/command.rs |
| [#2386](https://github.com/apache/iggy/issues/2386) Transfer TCP connections across shards (only for `PollMessages`/`SendMessages` commands) | core/server | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2396](https://github.com/apache/iggy/issues/2396) [Nodejs SDK] should Client expose  event handler for events through on/once | foreign/node | claimed | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2517](https://github.com/apache/iggy/issues/2517) End-to-end test suite for Apache Iggy Web UI | web ui | assigned | -1 | 2 | Add one scenario/test + wire into harness. |  |
| [#2524](https://github.com/apache/iggy/issues/2524) Figure out how to extract `IggyError` into an dedicated crate. | core/other | unassigned | 0 | 0 | Investigate surface, propose small first PR. |  |
| [#2562](https://github.com/apache/iggy/issues/2562) Clarify Distributed Clustering Status and Non-Linux Platform Support | core/other | assigned | -3 | 3 | Investigate surface, propose small first PR. |  |
| [#2590](https://github.com/apache/iggy/issues/2590) Remove overreliance on Identifier from server | core/server | unassigned | 0 | 0 | Investigate surface, propose small first PR. |  |
| [#2628](https://github.com/apache/iggy/issues/2628) [csharp SDK] Implement leader_redirection scenario in BDD tests | foreign/csharp | assigned | -1 | 2 | Add one scenario/test + wire into harness. |  |
| [#2687](https://github.com/apache/iggy/issues/2687) Implement partitions replicated log. | core/other | assigned | -3 | 2 | Write plan/RFC, split into sub-issues, then implement. |  |
| [#2699](https://github.com/apache/iggy/issues/2699) Normalize enum serialization strategy | core/other | unassigned | 0 | 0 | Investigate surface, propose small first PR. |  |
| [#2715](https://github.com/apache/iggy/issues/2715) Non-deterministic consumer offset jump to "latest" on large streams (~50M records) | core/other | unassigned | 0 | 3 | Reproduce + regression test + fix. |  |
| [#2763](https://github.com/apache/iggy/issues/2763) [C++ SDK] Create Rust side FFI bindings for the C++ SDK | foreign/cpp + Rust FFI | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2764](https://github.com/apache/iggy/issues/2764) [C++ SDK] Create high level client from the generated bindings. | foreign/cpp + Rust FFI | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2776](https://github.com/apache/iggy/issues/2776) [Python SDK] Create `sync-python-version.sh` to keep python versions in sync across folders and docker images | foreign/python | claimed | 1 | 0 | Add script/workflow check + CI gating. | .github/workflows/_build_python_wheels.yml, .github/workflows/_common.yml, scripts/ci/sync-python-version.sh |
| [#2827](https://github.com/apache/iggy/issues/2827) Replace `DashMap` with `papaya` for `shards_table` in server shard | core/other | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2830](https://github.com/apache/iggy/issues/2830) [C++ SDK] Consider converting high-level classes in iggy.hpp to rust-based enums | foreign/cpp + Rust FFI | unassigned | 0 | 0 | Investigate surface, propose small first PR. |  |
| [#2835](https://github.com/apache/iggy/issues/2835) feat(python): Add QUIC, HTTP, and WebSocket Transport Protocol Support to Python SDK | core/* | assigned | 3 | 0 | Investigate surface, propose small first PR. | core/common/src/types/configuration/auth_config/connection_string.rs, core/common/src/types/configuration/transport.rs, core/integration/tests/server/cg.rs |
| [#2872](https://github.com/apache/iggy/issues/2872) Add integration test for message deduplication | core/server | assigned | 1 | 2 | Add one scenario/test + wire into harness. | core/common/src/deduplication/message_deduplicator.rs, core/common/src/types/message/messages_batch_mut.rs, core/server/src/streaming/partitions/helpers.rs |
| [#2883](https://github.com/apache/iggy/issues/2883) Go SDK: Add unit tests for command serialization/deserialization | foreign/go | assigned | -1 | 2 | Add one scenario/test + wire into harness. |  |
| [#2965](https://github.com/apache/iggy/issues/2965) [C++ SDK] implement `bdd/scenarios/basic_messaging.feature` test for C++ SDK | foreign/cpp + Rust FFI | assigned | 1 | 2 | Add script/workflow check + CI gating. | scripts/run-bdd-tests.sh |
| [#2978](https://github.com/apache/iggy/issues/2978) Increase Go Codecov Coverage | core/other | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#2981](https://github.com/apache/iggy/issues/2981) Go SDK: CreateUser omits mandatory permissions_len field when permissions are nil | foreign/go | claimed | 3 | 0 | Small bugfix + regression test. | core/binary_protocol/src/requests/users/create_user.rs, foreign/go/internal/command/user.go |
| [#2982](https://github.com/apache/iggy/issues/2982) Go SDK: UpdatePermissions omits mandatory `permissions_len` field when permissions are nil | foreign/go | assigned | 3 | 0 | Small bugfix + regression test. | core/binary_protocol/src/requests/users/update_permissions.rs, foreign/go/internal/command/user.go |
| [#2984](https://github.com/apache/iggy/issues/2984) Provide a Cross-Language Conformance Testing for serizalize/deserialize logic of SDK | core/other | unassigned | -3 | 0 | Confirm scope/timing; likely defer. |  |
| [#2985](https://github.com/apache/iggy/issues/2985) auto generate helm readme so that drift between document and code remains at minimum + beautify(fix formatting issues) YAML | core/other | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#3000](https://github.com/apache/iggy/issues/3000) (cli): Add `context show` and `session status` commands to CLI | CLI | unassigned | 0 | 0 | Investigate surface, propose small first PR. |  |
| [#3012](https://github.com/apache/iggy/issues/3012) NixOS packages and Module. | packaging (nix) | assigned | -1 | 0 | Investigate surface, propose small first PR. |  |
| [#3030](https://github.com/apache/iggy/issues/3030) (cli): `context use` can fail on fresh system when `~/.iggy/` directory does not exist | CLI | unassigned | 4 | 0 | Small bugfix + regression test. |  |
| [#3075](https://github.com/apache/iggy/issues/3075) Address validation for `QuicClientConfigBuilder` and `HttpClientConfigBuilder` | core/common config | unassigned | 0 | 0 | Small bugfix + regression test. |  |

# Non-Connector Open Issues (v2): Surface + Status + Starter Notes

Generated from `/tmp/iggy-current-open-issues.json` at 2026-04-09T09:27:15Z.

## Counts
- Total open issues snapshot: 78
- Non-connector open issues: 58
- Status: {'claimed': 11, 'assigned': 31, 'unassigned': 16}
- Type: {'other': 30, 'docs': 2, 'test': 12, 'feature': 13, 'bug': 1}
- Surface: {'Node SDK': 2, 'Core/Other': 19, 'CLI': 2, 'Docs': 2, 'Server': 9, 'Server metrics': 1, 'C++ SDK': 5, 'Packaging': 1, 'C# SDK': 1, 'Go SDK': 4, 'Testing': 1, 'Python SDK': 2, 'Java SDK': 8, 'Web UI': 1}

## Table (updatedAt desc)

| issue | surface | type | status | gfi | notes |
| --- | --- | --- | --- | --- | --- |
| [#2396](https://github.com/apache/iggy/issues/2396) [Nodejs SDK] should Client expose  event handler for events through on/once | Node SDK | other | claimed | yes |  |
| [#3075](https://github.com/apache/iggy/issues/3075) Address validation for `QuicClientConfigBuilder` and `HttpClientConfigBuilder` | Core/Other | other | claimed |  | Follow-up to PR #2923; no PR closing it yet. |
| [#3030](https://github.com/apache/iggy/issues/3030) (cli): `context use` can fail on fresh system when `~/.iggy/` directory does not exist | CLI | other | claimed |  | Has open PR #3069 (as of 2026-04-09). |
| [#2985](https://github.com/apache/iggy/issues/2985) auto generate helm readme so that drift between document and code remains at minimum + beautify(fix formatting issues) YAML | Docs | docs | assigned |  |  |
| [#2872](https://github.com/apache/iggy/issues/2872) Add integration test for message deduplication | Server | test | assigned | yes |  |
| [#1420](https://github.com/apache/iggy/issues/1420) Implement Direct I/O, bypass kernel page cache | Core/Other | feature | assigned |  |  |
| [#2699](https://github.com/apache/iggy/issues/2699) Normalize enum serialization strategy | Core/Other | other | unassigned |  | Had PR #3051 (closed, not merged) due to wire-compat concerns; not a trivial change. |
| [#3000](https://github.com/apache/iggy/issues/3000) (cli): Add `context show` and `session status` commands to CLI | CLI | other | unassigned |  | Related: merged PR #2998 added context create/delete; still missing context show + session status. |
| [#19](https://github.com/apache/iggy/issues/19) Implement & expose custom server metrics for Prometheus | Server metrics | feature | claimed | yes |  |
| [#2965](https://github.com/apache/iggy/issues/2965) [C++ SDK] implement `bdd/scenarios/basic_messaging.feature` test for C++ SDK | C++ SDK | test | assigned | yes |  |
| [#3012](https://github.com/apache/iggy/issues/3012) NixOS packages and Module. | Packaging | other | assigned |  |  |
| [#2628](https://github.com/apache/iggy/issues/2628) [csharp SDK] Implement leader_redirection scenario in BDD tests | C# SDK | test | assigned | yes |  |
| [#469](https://github.com/apache/iggy/issues/469) Write fuzzing testcases for `iggy-server` | Server | test | assigned | yes |  |
| [#2984](https://github.com/apache/iggy/issues/2984) Provide a Cross-Language Conformance Testing for serizalize/deserialize logic of SDK | Core/Other | test | unassigned |  |  |
| [#2981](https://github.com/apache/iggy/issues/2981) Go SDK: CreateUser omits mandatory permissions_len field when permissions are nil | Go SDK | other | claimed |  | Claimed in comments; PR #3015 merged but explicitly did not fix; still open. |
| [#2982](https://github.com/apache/iggy/issues/2982) Go SDK: UpdatePermissions omits mandatory `permissions_len` field when permissions are nil | Go SDK | other | assigned |  | Assigned; PR #3015 merged but explicitly did not fix; still open. |
| [#2978](https://github.com/apache/iggy/issues/2978) Increase Go Codecov Coverage | Core/Other | other | assigned |  |  |
| [#2715](https://github.com/apache/iggy/issues/2715) Non-deterministic consumer offset jump to "latest" on large streams (~50M records) | Core/Other | bug | claimed |  |  |
| [#2148](https://github.com/apache/iggy/issues/2148) Add more BDD test scenarios. | Testing | test | unassigned | yes |  |
| [#2146](https://github.com/apache/iggy/issues/2146) Add Migration Guides to Docs | Docs | docs | assigned | yes |  |
| [#477](https://github.com/apache/iggy/issues/477) Write wireshark dissector for `iggy` protocol | Core/Other | other | assigned | yes |  |
| [#2776](https://github.com/apache/iggy/issues/2776) [Python SDK] Create `sync-python-version.sh` to keep python versions in sync across folders and docker images | Python SDK | other | claimed | yes |  |
| [#2883](https://github.com/apache/iggy/issues/2883) Go SDK: Add unit tests for command serialization/deserialization | Go SDK | test | assigned |  |  |
| [#1594](https://github.com/apache/iggy/issues/1594) Explore using arrayvec/arraystring crate for stack allocated strings | Server | other | unassigned |  |  |
| [#88](https://github.com/apache/iggy/issues/88) Add feature flag for conditional compilation of protocols (TCP, QUIC, HTTP) | Core/Other | feature | assigned | yes |  |
| [#2835](https://github.com/apache/iggy/issues/2835) feat(python): Add QUIC, HTTP, and WebSocket Transport Protocol Support to Python SDK | Python SDK | feature | assigned |  |  |
| [#2830](https://github.com/apache/iggy/issues/2830) [C++ SDK] Consider converting high-level classes in iggy.hpp to rust-based enums | C++ SDK | other | unassigned |  |  |
| [#2827](https://github.com/apache/iggy/issues/2827) Replace `DashMap` with `papaya` for `shards_table` in server shard | Server | other | assigned |  |  |
| [#2100](https://github.com/apache/iggy/issues/2100) Implement C++ SDK | C++ SDK | feature | assigned |  |  |
| [#2763](https://github.com/apache/iggy/issues/2763) [C++ SDK] Create Rust side FFI bindings for the C++ SDK | C++ SDK | other | assigned |  |  |
| [#2764](https://github.com/apache/iggy/issues/2764) [C++ SDK] Create high level client from the generated bindings. | C++ SDK | other | assigned |  |  |
| [#2205](https://github.com/apache/iggy/issues/2205) [Java SDK] Add comprehensive offset management for consumer groups | Java SDK | other | assigned |  |  |
| [#18](https://github.com/apache/iggy/issues/18) Server-side message compression feature | Server | other | assigned |  |  |
| [#21](https://github.com/apache/iggy/issues/21) Configurable message size threshold | Core/Other | other | assigned | yes |  |
| [#1714](https://github.com/apache/iggy/issues/1714) Implement synchronous SDK client | Core/Other | feature | unassigned |  |  |
| [#200](https://github.com/apache/iggy/issues/200) PGO in CI | Server | other | unassigned |  |  |
| [#2687](https://github.com/apache/iggy/issues/2687) Implement partitions replicated log. | Core/Other | feature | assigned |  |  |
| [#2228](https://github.com/apache/iggy/issues/2228) [Java SDK] Add Performance Benchmarks for Java SDK | Java SDK | test | claimed |  |  |
| [#2203](https://github.com/apache/iggy/issues/2203) [Java SDK] Add batch send/receive operations for better throughput | Java SDK | other | assigned |  |  |
| [#2210](https://github.com/apache/iggy/issues/2210) [Java SDK] Add AutoCloseable support and proper cleanup | Java SDK | other | assigned |  |  |
| [#2206](https://github.com/apache/iggy/issues/2206) [Java SDK] Implement pluggable serialization framework | Java SDK | other | assigned |  |  |
| [#2562](https://github.com/apache/iggy/issues/2562) Clarify Distributed Clustering Status and Non-Linux Platform Support | Core/Other | other | assigned |  |  |
| [#2590](https://github.com/apache/iggy/issues/2590) Remove overreliance on Identifier from server | Server | other | claimed |  |  |
| [#2524](https://github.com/apache/iggy/issues/2524) Figure out how to extract `IggyError` into an dedicated crate. | Core/Other | other | unassigned |  |  |
| [#2517](https://github.com/apache/iggy/issues/2517) End-to-end test suite for Apache Iggy Web UI | Web UI | test | assigned |  |  |
| [#2386](https://github.com/apache/iggy/issues/2386) Transfer TCP connections across shards (only for `PollMessages`/`SendMessages` commands) | Server | other | assigned |  |  |
| [#46](https://github.com/apache/iggy/issues/46) Add handling of maximum size of `logs` folder | Core/Other | feature | assigned | yes |  |
| [#1914](https://github.com/apache/iggy/issues/1914) Implement Clustering (VSR) | Core/Other | feature | claimed |  |  |
| [#2373](https://github.com/apache/iggy/issues/2373) Implement generic Message Header system for Consensus | Server | feature | assigned |  |  |
| [#1890](https://github.com/apache/iggy/issues/1890) Add Node.js SDK examples | Node SDK | feature | assigned | yes |  |
| [#1969](https://github.com/apache/iggy/issues/1969) Add official Ruby sdk | Core/Other | feature | claimed |  |  |
| [#2207](https://github.com/apache/iggy/issues/2207) Schema registry for data governance | Core/Other | other | unassigned |  |  |
| [#2209](https://github.com/apache/iggy/issues/2209) [Java SDK] Implement metrics collection and monitoring | Java SDK | other | unassigned |  |  |
| [#2226](https://github.com/apache/iggy/issues/2226) [Java SDK] Add Error Handling and Edge Case Tests for Async Client | Java SDK | test | unassigned |  |  |
| [#2229](https://github.com/apache/iggy/issues/2229) [Java SDK] Implement Load Testing Suite | Java SDK | test | unassigned |  |  |
| [#1986](https://github.com/apache/iggy/issues/1986) Rewrite golang BDD tests to use scenarios | Go SDK | test | unassigned |  | Looks already solved (Go BDD suite uses bdd/scenarios); likely needs a close-comment. |
| [#1710](https://github.com/apache/iggy/issues/1710) [ feature request ] Feature to replace dependency on libdbus-sys via keyring | Core/Other | other | unassigned |  |  |
| [#1419](https://github.com/apache/iggy/issues/1419) Implement Tiered-Storage | Core/Other | feature | unassigned |  |  |

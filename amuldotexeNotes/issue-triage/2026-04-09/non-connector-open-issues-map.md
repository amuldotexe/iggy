# Non-Connector Open Issues: Surface, Simplicity, Value

Generated from `/tmp/iggy-current-open-issues.json` on 2026-04-09T09:11:18Z (UTC).

## Legend
- Complexity: `S/M`, `M`, `M/L`, `L`, `XL` (rough estimate)
- Value: `H`, `M/H`, `M` (rough estimate)
- Ownership: `assigned`, `claimed-in-comments`, `unassigned`
- All estimates are heuristic; use issue body as source of truth.

## Quick Counts
- Non-connector open issues: **58**
- Ownership: **assigned**=31, **claimed-in-comments**=5, **unassigned**=22
- Top surfaces:
  - Core/Other (needs pinpointing): 16
  - Server (core/server): 13
  - Java SDK (foreign/java): 8
  - C++ SDK (foreign/cpp + Rust FFI): 5
  - Go SDK (foreign/go): 3
  - CLI (core/cli): 2
  - Python SDK (foreign/python, bdd/python, examples/python): 2
  - Node SDK (foreign/node): 1
  - Core config builders (core/common): 1
  - Server metrics (core/server, core/http): 1
  - Packaging (nix): 1
  - C# SDK (foreign/csharp): 1

## Full Map (Most Recently Updated First)

| issue | surface | ownership | complexity | value | PR hint | likely first step |
| --- | --- | --- | --- | --- | --- | --- |
| [#2396](https://github.com/apache/iggy/issues/2396) [Nodejs SDK] should Client expose  event handler for events through on/once | Node SDK (foreign/node) | claimed-in-comments | M/L | M | no | Comment to coordinate before starting. |
| [#3075](https://github.com/apache/iggy/issues/3075) Address validation for `QuicClientConfigBuilder` and `HttpClientConfigBuilder` | Core config builders (core/common) | unassigned | S/M | M | yes | Add a regression test; implement the smallest fix. |
| [#3030](https://github.com/apache/iggy/issues/3030) (cli): `context use` can fail on fresh system when `~/.iggy/` directory does not exist | CLI (core/cli) | unassigned | S/M | M | yes | Add a regression test; implement the smallest fix. |
| [#2985](https://github.com/apache/iggy/issues/2985) auto generate helm readme so that drift between document and code remains at minimum + beautify(fix formatting issues) YAML | Core/Other (needs pinpointing) | assigned | M/L | M | yes | Comment to coordinate before starting. |
| [#2872](https://github.com/apache/iggy/issues/2872) Add integration test for message deduplication | Server (core/server) | assigned | M | M/H | no | Comment to coordinate before starting. |
| [#1420](https://github.com/apache/iggy/issues/1420) Implement Direct I/O, bypass kernel page cache | Core/Other (needs pinpointing) | assigned | XL | H | no | Comment to coordinate before starting. |
| [#2699](https://github.com/apache/iggy/issues/2699) Normalize enum serialization strategy | Core/Other (needs pinpointing) | unassigned | M/L | M | yes | Locate code via ripgrep; propose a small first PR. |
| [#3000](https://github.com/apache/iggy/issues/3000) (cli): Add `context show` and `session status` commands to CLI | CLI (core/cli) | unassigned | M/L | M | no | Locate code via ripgrep; propose a small first PR. |
| [#19](https://github.com/apache/iggy/issues/19) Implement & expose custom server metrics for Prometheus | Server metrics (core/server, core/http) | claimed-in-comments | L | H | yes | Comment to coordinate before starting. |
| [#2965](https://github.com/apache/iggy/issues/2965) [C++ SDK] implement `bdd/scenarios/basic_messaging.feature` test for C++ SDK | C++ SDK (foreign/cpp + Rust FFI) | assigned | M | M/H | no | Comment to coordinate before starting. |
| [#3012](https://github.com/apache/iggy/issues/3012) NixOS packages and Module. | Packaging (nix) | assigned | M/L | M | yes | Comment to coordinate before starting. |
| [#2628](https://github.com/apache/iggy/issues/2628) [csharp SDK] Implement leader_redirection scenario in BDD tests | C# SDK (foreign/csharp) | assigned | M | M | yes | Comment to coordinate before starting. |
| [#469](https://github.com/apache/iggy/issues/469) Write fuzzing testcases for `iggy-server` | Server (core/server) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#2984](https://github.com/apache/iggy/issues/2984) Provide a Cross-Language Conformance Testing for serizalize/deserialize logic of SDK | Core/Other (needs pinpointing) | unassigned | XL (postponed) | M | no | Confirm it is still wanted; otherwise defer. |
| [#2981](https://github.com/apache/iggy/issues/2981) Go SDK: CreateUser omits mandatory permissions_len field when permissions are nil | Go SDK (foreign/go) | claimed-in-comments | S/M | M | no | Comment to coordinate before starting. |
| [#2982](https://github.com/apache/iggy/issues/2982) Go SDK: UpdatePermissions omits mandatory `permissions_len` field when permissions are nil | Go SDK (foreign/go) | assigned | S/M | M | no | Comment to coordinate before starting. |
| [#2978](https://github.com/apache/iggy/issues/2978) Increase Go Codecov Coverage | Core/Other (needs pinpointing) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#2715](https://github.com/apache/iggy/issues/2715) Non-deterministic consumer offset jump to "latest" on large streams (~50M records) | Core/Other (needs pinpointing) | unassigned | M | H | no | Reproduce locally; add a failing test if possible. |
| [#2148](https://github.com/apache/iggy/issues/2148) Add more BDD test scenarios. | Core/Other (needs pinpointing) | unassigned | M | M | no | Add one minimal scenario/test; keep PR small. |
| [#2146](https://github.com/apache/iggy/issues/2146) Add Migration Guides to Docs | Docs (docs/, README) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#477](https://github.com/apache/iggy/issues/477) Write wireshark dissector for `iggy` protocol | Core/Other (needs pinpointing) | assigned | XL (postponed) | M | no | Confirm it is still wanted; otherwise defer. |
| [#2776](https://github.com/apache/iggy/issues/2776) [Python SDK] Create `sync-python-version.sh` to keep python versions in sync across folders and docker images | Python SDK (foreign/python, bdd/python, examples/python) | claimed-in-comments | M/L | M/H | no | Comment to coordinate before starting. |
| [#2883](https://github.com/apache/iggy/issues/2883) Go SDK: Add unit tests for command serialization/deserialization | Go SDK (foreign/go) | assigned | M | M | no | Comment to coordinate before starting. |
| [#1594](https://github.com/apache/iggy/issues/1594) Explore using arrayvec/arraystring crate for stack allocated strings | Server (core/server) | unassigned | XL (postponed) | H | yes | Confirm it is still wanted; otherwise defer. |
| [#88](https://github.com/apache/iggy/issues/88) Add feature flag for conditional compilation of protocols (TCP, QUIC, HTTP) | Server (core/server) | assigned | XL (postponed) | M | no | Confirm it is still wanted; otherwise defer. |
| [#2835](https://github.com/apache/iggy/issues/2835) feat(python): Add QUIC, HTTP, and WebSocket Transport Protocol Support to Python SDK | Python SDK (foreign/python, bdd/python, examples/python) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#2830](https://github.com/apache/iggy/issues/2830) [C++ SDK] Consider converting high-level classes in iggy.hpp to rust-based enums | C++ SDK (foreign/cpp + Rust FFI) | unassigned | M/L | M | no | Locate code via ripgrep; propose a small first PR. |
| [#2827](https://github.com/apache/iggy/issues/2827) Replace `DashMap` with `papaya` for `shards_table` in server shard | Server (core/server) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#2100](https://github.com/apache/iggy/issues/2100) Implement C++ SDK | C++ SDK (foreign/cpp + Rust FFI) | assigned | L | H | yes | Comment to coordinate before starting. |
| [#2763](https://github.com/apache/iggy/issues/2763) [C++ SDK] Create Rust side FFI bindings for the C++ SDK | C++ SDK (foreign/cpp + Rust FFI) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#2764](https://github.com/apache/iggy/issues/2764) [C++ SDK] Create high level client from the generated bindings. | C++ SDK (foreign/cpp + Rust FFI) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#2205](https://github.com/apache/iggy/issues/2205) [Java SDK] Add comprehensive offset management for consumer groups | Java SDK (foreign/java) | assigned | M/L | M | yes | Comment to coordinate before starting. |
| [#18](https://github.com/apache/iggy/issues/18) Server-side message compression feature | Core/Other (needs pinpointing) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#21](https://github.com/apache/iggy/issues/21) Configurable message size threshold | Core/Other (needs pinpointing) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#1714](https://github.com/apache/iggy/issues/1714) Implement synchronous SDK client | Rust SDK (core/sdk, core/common) | unassigned | L | H | yes | Write a short plan comment; split into sub-tasks. |
| [#200](https://github.com/apache/iggy/issues/200) PGO in CI | Server (core/server) | unassigned | M/L | M | no | Locate code via ripgrep; propose a small first PR. |
| [#2687](https://github.com/apache/iggy/issues/2687) Implement partitions replicated log. | Server (core/server) | assigned | XL | H | no | Comment to coordinate before starting. |
| [#2228](https://github.com/apache/iggy/issues/2228) [Java SDK] Add Performance Benchmarks for Java SDK | Java SDK (foreign/java) | claimed-in-comments | L | M/H | no | Comment to coordinate before starting. |
| [#2203](https://github.com/apache/iggy/issues/2203) [Java SDK] Add batch send/receive operations for better throughput | Java SDK (foreign/java) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#2210](https://github.com/apache/iggy/issues/2210) [Java SDK] Add AutoCloseable support and proper cleanup | Java SDK (foreign/java) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#2206](https://github.com/apache/iggy/issues/2206) [Java SDK] Implement pluggable serialization framework | Java SDK (foreign/java) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#2562](https://github.com/apache/iggy/issues/2562) Clarify Distributed Clustering Status and Non-Linux Platform Support | Server (core/server) | assigned | M/L | M | yes | Comment to coordinate before starting. |
| [#2590](https://github.com/apache/iggy/issues/2590) Remove overreliance on Identifier from server | Server (core/server) | unassigned | M/L | M | yes | Locate code via ripgrep; propose a small first PR. |
| [#2524](https://github.com/apache/iggy/issues/2524) Figure out how to extract `IggyError` into an dedicated crate. | Core/Other (needs pinpointing) | unassigned | M/L | M | no | Locate code via ripgrep; propose a small first PR. |
| [#2517](https://github.com/apache/iggy/issues/2517) End-to-end test suite for Apache Iggy Web UI | Web UI (repo UI surface) | assigned | L | M/H | no | Comment to coordinate before starting. |
| [#2386](https://github.com/apache/iggy/issues/2386) Transfer TCP connections across shards (only for `PollMessages`/`SendMessages` commands) | Server (core/server) | assigned | M/L | H | yes | Comment to coordinate before starting. |
| [#46](https://github.com/apache/iggy/issues/46) Add handling of maximum size of `logs` folder | Server (core/server) | assigned | M/L | M | no | Comment to coordinate before starting. |
| [#1914](https://github.com/apache/iggy/issues/1914) Implement Clustering (VSR) | Server (core/server) | unassigned | XL | H | no | Write a short plan comment; split into sub-tasks. |
| [#2373](https://github.com/apache/iggy/issues/2373) Implement generic Message Header system for Consensus | Server (core/server) | assigned | L | H | no | Comment to coordinate before starting. |
| [#1890](https://github.com/apache/iggy/issues/1890) Add Node.js SDK examples | CI/CD (.github/workflows, scripts/ci) | assigned | M/L | M | yes | Comment to coordinate before starting. |
| [#1969](https://github.com/apache/iggy/issues/1969) Add official Ruby sdk | Core/Other (needs pinpointing) | unassigned | M/L | M | no | Locate code via ripgrep; propose a small first PR. |
| [#2207](https://github.com/apache/iggy/issues/2207) Schema registry for data governance | Core/Other (needs pinpointing) | unassigned | XL | M | no | Write a short plan comment; split into sub-tasks. |
| [#2209](https://github.com/apache/iggy/issues/2209) [Java SDK] Implement metrics collection and monitoring | Java SDK (foreign/java) | unassigned | M/L | H | no | Locate code via ripgrep; propose a small first PR. |
| [#2226](https://github.com/apache/iggy/issues/2226) [Java SDK] Add Error Handling and Edge Case Tests for Async Client | Java SDK (foreign/java) | unassigned | M | M | no | Add one minimal scenario/test; keep PR small. |
| [#2229](https://github.com/apache/iggy/issues/2229) [Java SDK] Implement Load Testing Suite | Java SDK (foreign/java) | unassigned | L | M/H | no | Write a short plan comment; split into sub-tasks. |
| [#1986](https://github.com/apache/iggy/issues/1986) Rewrite golang BDD tests to use scenarios | Core/Other (needs pinpointing) | unassigned | M | M | no | Add one minimal scenario/test; keep PR small. |
| [#1710](https://github.com/apache/iggy/issues/1710) [ feature request ] Feature to replace dependency on libdbus-sys via keyring | Core/Other (needs pinpointing) | unassigned | M/L | M | no | Locate code via ripgrep; propose a small first PR. |
| [#1419](https://github.com/apache/iggy/issues/1419) Implement Tiered-Storage | Core/Other (needs pinpointing) | unassigned | XL | H | no | Write a short plan comment; split into sub-tasks. |

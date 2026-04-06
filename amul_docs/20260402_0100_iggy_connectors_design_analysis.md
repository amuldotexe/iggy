# Apache Iggy Connectors: Design Analysis & Competitive Positioning

**Author:** Deep Analysis (automated research)
**Date:** 2026-04-02
**Repository:** [apache/iggy](https://github.com/apache/iggy) at commit `411a697eb`
**Scope:** `core/connectors/` -- SDK, runtime, sinks, sources

---

## Executive Summary (SCQA Framework)

### Situation

Apache Iggy is a high-performance message streaming platform written in Rust, accepted into the Apache Incubator in February 2025. It supports QUIC, TCP, WebSocket, and HTTP transport protocols and processes millions of messages per second with sub-millisecond P99 latency. As the platform matured, a connector framework became necessary to bridge Iggy with external data systems -- databases, search engines, object stores, and other message platforms.

### Complication

Building a connector framework for a Rust-native streaming platform presents a unique engineering tension:

1. **Performance vs. Extensibility**: Iggy's core value proposition is extreme performance via Rust's zero-cost abstractions, `io_uring`, and shared-nothing architecture. A connector framework must not compromise this.
2. **Safety vs. Dynamism**: Rust's static type system and memory safety guarantees conflict with the need to dynamically load third-party plugins at runtime.
3. **Ecosystem Gap**: Kafka Connect has hundreds of connectors and a decade of ecosystem investment. Pulsar IO has ~20+. A new entrant must offer a compelling architectural story to attract connector developers.
4. **Process Isolation vs. Latency**: Embedding plugins risks crashing the core server; isolating them adds serialization and IPC overhead.

### Question

How should Apache Iggy design a connector system that preserves its performance characteristics, leverages Rust's type safety, enables third-party extensibility, and positions itself competitively against mature alternatives?

### Answer

Iggy chose a **dynamically-loaded shared library architecture** with C FFI boundaries, where:

- Connectors are compiled as native Rust shared libraries (`.so`/`.dylib`/`.dll`)
- The runtime loads them via `dlopen2` at startup, calling well-defined C ABI functions
- Data crosses the FFI boundary via `postcard` (compact binary serialization)
- Each plugin receives its own Tokio runtime for isolation
- The runtime runs as a **separate process** from the Iggy server, preventing plugin crashes from affecting the core streaming engine
- A three-layer architecture (SDK / Runtime / Connectors) separates concerns cleanly

This design achieves near-native performance for connectors while maintaining process-level isolation, type safety within the Rust ecosystem, and a simple developer experience through macros that hide the FFI complexity.

---

# Key Finding 1: Design Philosophy -- "Statically Typed, Dynamically Loaded"

The central design philosophy is captured in the README's own words: *"The highly performant and modular runtime for statically typed, yet dynamically loaded connectors."* This phrase encodes a deliberate resolution of the Rust plugin paradox.

## 1.1 Process Isolation as a First Principle

The earliest design discussion ([GitHub Discussion #1670](https://github.com/apache/iggy/discussions/1670)) explicitly articulated the reasoning for connector process isolation:

> "Having separate processes, instead of UDFs directly embedded/invoked by the server, makes it easier to deal with potential errors (an invalid plugin won't crash the streaming server)."

This is a critical architectural choice that differentiates Iggy from systems like Kafka Connect (where connectors run in the same JVM as the Connect worker). The connector runtime (`iggy-connectors` binary) is a completely separate process from the Iggy server. It connects to Iggy as a regular TCP client using `IggyClient`, `IggyProducer`, and `IggyConsumer`.

**Implications:**
- A buggy connector cannot corrupt Iggy's internal state or cause the server to crash
- The connector runtime can be scaled, restarted, and upgraded independently
- The memory allocator (`mimalloc`) and async runtime (Tokio) are chosen specifically for the connector workload, independent of the server's `io_uring`/`compio` architecture

## 1.2 C FFI as the Stability Contract

Rust lacks a stable ABI -- struct layouts can change between compiler versions, or even between compilations. The team chose the C ABI as the stability boundary:

```rust
// From sdk/src/sink.rs -- the FFI functions exposed by sink_connector! macro
#[unsafe(no_mangle)]
unsafe extern "C" fn iggy_sink_open(id: u32, config_ptr: *const u8, config_len: usize, log_callback: LogCallback) -> i32
#[unsafe(no_mangle)]
unsafe extern "C" fn iggy_sink_consume(id: u32, topic_meta_ptr: *const u8, ...) -> i32
#[unsafe(no_mangle)]
unsafe extern "C" fn iggy_sink_close(id: u32) -> i32
#[unsafe(no_mangle)]
extern "C" fn iggy_sink_version() -> *const std::ffi::c_char
```

The FFI boundary uses only C-compatible types: `u32`, raw pointers (`*const u8`), `usize`, and `i32` return codes. Complex Rust types are serialized to bytes via `postcard` before crossing the boundary, then deserialized on the other side. This makes the interface version-resilient -- the binary wire format is the contract, not the Rust type layout.

**Key insight from the POSIX naming collision bug ([#2770](https://github.com/apache/iggy/issues/2770)):** The original FFI functions were named `open`, `close`, etc., which collided with POSIX system calls, causing segfaults when connectors accessed the filesystem. The fix ([PR #2771](https://github.com/apache/iggy/pull/2771)) renamed them to `iggy_sink_open`, `iggy_source_handle`, etc. -- a hard lesson about the C ABI namespace being truly global.

## 1.3 Macro-Hidden Complexity

The `sink_connector!` and `source_connector!` macros are the developer-facing abstraction that hides the FFI complexity:

```rust
// What a connector developer writes:
sink_connector!(StdoutSink);

// What the macro generates (simplified):
static INSTANCES: Lazy<DashMap<u32, SinkContainer<StdoutSink>>> = Lazy::new(DashMap::new);

extern "C" fn iggy_sink_open(id: u32, config_ptr: *const u8, ...) -> i32 {
    let mut container = SinkContainer::new(id);
    let result = container.open(id, config_ptr, config_len, log_callback, StdoutSink::new);
    INSTANCES.insert(id, container);
    result
}
// ... plus iggy_sink_consume, iggy_sink_close, iggy_sink_version
```

The macro also generates a compile-time trait assertion (`fn assert_trait<T: Sink>() {}`), ensuring the type passed to the macro actually implements `Sink` or `Source`. This catches errors at compile time rather than at runtime FFI invocation.

**Developer experience impact:** A new connector can be created by implementing the 3-method `Sink` or `Source` trait, calling the macro, and providing a `new()` constructor. The boilerplate of FFI, instance management, serialization, and Tokio runtime bootstrapping is entirely hidden.

---

# Key Finding 2: Architecture & Code Structure -- Three-Layer Separation

## 2.1 The SDK Layer (`core/connectors/sdk/`)

The SDK is the **shared contract** between the runtime and plugins. It contains:

| Component | Purpose |
|---|---|
| `Source` trait | 3 methods: `open()`, `poll()`, `close()` |
| `Sink` trait | 3 methods: `open()`, `consume()`, `close()` |
| `sink_connector!` / `source_connector!` macros | Generate FFI exports, instance management, logging |
| `StreamDecoder` / `StreamEncoder` traits | Schema-aware message (de)serialization |
| `Transform` trait | Per-message field transformations |
| `ConnectorState` | Serializable state wrapper for resumable connectors |
| `Payload` enum | `Json`, `Raw`, `Text`, `Proto`, `FlatBuffer` variants |
| `decoders/` and `encoders/` | Implementations for JSON, Raw, Text, Protobuf, FlatBuffers |
| `transforms/` | AddFields, DeleteFields, FilterFields, UpdateFields, ProtoConvert, FlatBufferConvert |

**Key design decision -- Payload as an enum, not a trait:**

```rust
pub enum Payload {
    Json(simd_json::OwnedValue),
    Raw(Vec<u8>),
    Text(String),
    Proto(String),
    FlatBuffer(Vec<u8>),
}
```

Using an enum rather than a trait object means the runtime can pattern-match on the payload type without dynamic dispatch. The `simd_json::OwnedValue` for JSON payloads specifically provides SIMD-accelerated parsing, consistent with Iggy's performance-first philosophy.

**Schema-to-codec mapping is centralized:**

```rust
impl Schema {
    pub fn decoder(self) -> Arc<dyn StreamDecoder> {
        match self {
            Schema::Json => Arc::new(JsonStreamDecoder),
            Schema::Raw => Arc::new(RawStreamDecoder),
            Schema::Text => Arc::new(TextStreamDecoder),
            Schema::Proto => Arc::new(ProtoStreamDecoder::default()),
            Schema::FlatBuffer => Arc::new(FlatBufferStreamDecoder::default()),
        }
    }
}
```

This means the runtime configuration (`schema = "json"` in TOML) directly determines the decoder/encoder pipeline, without the plugin needing to know about it.

## 2.2 The Runtime Layer (`core/connectors/runtime/`)

The runtime is the **orchestrator** -- a standalone binary that:

1. **Loads configuration** from local TOML files or a remote HTTP API
2. **Resolves and loads plugins** via `dlopen2` with platform-aware path resolution
3. **Manages the lifecycle** of each connector (Starting -> Running -> Stopping -> Stopped -> Error)
4. **Bridges data** between Iggy streams and plugin FFI calls
5. **Applies transforms** in the runtime process (not inside plugins)
6. **Persists state** for source connectors to enable resumable consumption
7. **Exposes an HTTP API** for monitoring, metrics, configuration management
8. **Reports Prometheus metrics** per-connector

**The data flow for a source connector:**

```
Plugin.poll() -> [postcard serialize] -> FFI callback -> [postcard deserialize]
  -> Runtime transform pipeline -> StreamEncoder.encode() -> IggyProducer.send()
  -> StateStorage.save()
```

**The data flow for a sink connector:**

```
IggyConsumer.next() -> batch accumulation -> StreamDecoder.decode()
  -> Runtime transform pipeline -> [postcard serialize] -> FFI callback
  -> Plugin.consume()
```

**Transforms run in the runtime, not in plugins.** This is architecturally significant: it means transforms are applied in a controlled, observable context with access to metrics and error handling, rather than inside an opaque shared library.

## 2.3 The Connector Layer (`sinks/` and `sources/`)

Each connector is a standalone Rust library crate compiled as `cdylib`:

**Available sinks (7):**
| Sink | External System | Notable Feature |
|---|---|---|
| `stdout_sink` | Terminal | Debug/development tool |
| `quickwit_sink` | Quickwit search engine | HTTP-based ingest |
| `postgres_sink` | PostgreSQL | Schema-aware inserts |
| `elasticsearch_sink` | Elasticsearch | Index management |
| `iceberg_sink` | Apache Iceberg (via REST catalog) | Dynamic/static routing, S3/GCS/Azure storage |
| `mongodb_sink` | MongoDB | Duplicate-key handling |
| `influxdb_sink` | InfluxDB | Time-series writes |
| `http_sink` | Any HTTP endpoint | Generic HTTP POST/PUT |

**Available sources (4):**
| Source | External System | Notable Feature |
|---|---|---|
| `random_source` | Synthetic data | Testing and benchmarking |
| `postgres_source` | PostgreSQL | CDC (WAL), polling, delete-after-read, mark-as-processed |
| `elasticsearch_source` | Elasticsearch | Timestamp-based tracking |
| `influxdb_source` | InfluxDB | Time-series reads |

**PostgreSQL source is the most sophisticated connector** -- it demonstrates the full power of the framework:
- Two modes: `polling` (query-based) and `cdc` (WAL logical replication)
- Custom query support with parameter substitution (`$table`, `$offset`, `$limit`, `$now`)
- State persistence via `ConnectorState` serialized with MessagePack
- Retry logic with exponential backoff for transient database errors
- Connection string redaction for security logging
- Comprehensive type mapping (15+ PostgreSQL types to JSON)

## 2.4 Configuration Architecture

The configuration system is notably sophisticated, with two providers:

1. **Local file provider**: Each connector has its own TOML file in a directory
2. **HTTP provider**: Fetches configs from a REST API with customizable URL templates, retry logic, and response extraction

Configuration supports:
- **Environment variable overrides**: `IGGY_CONNECTORS_SINK_STDOUT_ENABLED=false`
- **Config versioning**: Each config has a `version` field; the runtime API supports activating specific versions
- **Plugin-specific config**: The `plugin_config` section is opaque JSON passed directly to the plugin's `new()` constructor
- **Hot reload**: PR #2781 added the ability to restart connectors with new configs without restarting the entire runtime

---

# Key Finding 3: Evolution & Decision History

## 3.1 Timeline of Architectural Development

| Date | Commit/PR | Milestone |
|---|---|---|
| 2025-05 | [PR #1826](https://github.com/apache/iggy/pull/1826) `ac44ad3b4` | **Genesis**: Initial connectors runtime with SDK, Quickwit sink, Stdout sink, Test source. 3,960 lines added. |
| 2025-06 | [PR #1836](https://github.com/apache/iggy/pull/1836) `ae1470cdf` | Documentation and example plugins |
| 2025-06 | [PR #1863](https://github.com/apache/iggy/pull/1863) `9cad36abd` | Extended JSON field transformations |
| 2025-06 | [PR #1875](https://github.com/apache/iggy/pull/1875) `5d8f93b46` | Initial HTTP API for runtime |
| 2025-06 | [PR #1886](https://github.com/apache/iggy/pull/1886) `827abedaa` | Protobuf support |
| 2025-06 | [PR #1872](https://github.com/apache/iggy/pull/1872) `8dc210aec` | Elasticsearch sink and source |
| 2025-09 | [PR #1948](https://github.com/apache/iggy/pull/1948) `775508467` | State storage for source connectors |
| 2025-09 | [PR #1957](https://github.com/apache/iggy/pull/1957) `00619ac25` | FlatBuffers support |
| 2025-09 | [PR #1959](https://github.com/apache/iggy/pull/1959) `a541c9b89` | PostgreSQL sink and source |
| 2025-09 | [PR #2191](https://github.com/apache/iggy/pull/2191) `0d3ca8e40` | Apache Iceberg sink |
| 2025-11 | [PR #2317](https://github.com/apache/iggy/pull/2317) `e0322b2bf` | Split connector configs from runtime config |
| 2025-11 | [PR #2345](https://github.com/apache/iggy/pull/2345) `5d73772e4` | Configuration provider trait |
| 2025-12 | [PR #2401](https://github.com/apache/iggy/pull/2401) `734cb3430` | HTTP configuration provider |
| 2026-01 | [PR #2633](https://github.com/apache/iggy/pull/2633) `76e928a44` | Prometheus metrics and stats endpoints |
| 2026-02 | [PR #2685](https://github.com/apache/iggy/pull/2685) `0aa1f3864` | State and memory leak fixes, plugin enrichment |
| 2026-02 | [PR #2771](https://github.com/apache/iggy/pull/2771) `9ecbcc00d` | Fix POSIX FFI symbol collision |
| 2026-02 | [PR #2781](https://github.com/apache/iggy/pull/2781) `deb3eaad4` | Hot-reload: restart connectors without runtime restart |
| 2026-02 | [PR #2815](https://github.com/apache/iggy/pull/2815) `f91486031` | MongoDB sink connector |
| 2026-03 | [PR #2925](https://github.com/apache/iggy/pull/2925) `c9e40b693` | Generic HTTP sink connector |
| 2026-03 | [PR #2933](https://github.com/apache/iggy/pull/2933) `0460de9bd` | InfluxDB sink and source |

## 3.2 Key Architectural Decisions and Their Rationale

### Decision 1: dlopen2 over WASM or IPC

The original Discussion #1670 considered three isolation mechanisms:
- **IPC via iceoryx2**: Ultra-fast inter-process communication
- **Network-based**: TCP/UDP/QUIC/HTTP to remote connectors
- **Dynamic loading**: dlopen for in-process shared libraries

The team chose `dlopen2` for the initial implementation because:
- Zero IPC overhead -- function calls are direct, with only serialization cost
- Rust-to-Rust FFI preserves most of the type safety within the plugin
- The runtime process already provides crash isolation from the Iggy server
- WASM was considered "too much overhead in high-performance setups"

### Decision 2: postcard for FFI Serialization

Data crossing the FFI boundary uses `postcard`, a `#[no_std]`-compatible, zero-copy-capable binary serializer. This was chosen over:
- `serde_json`: Too slow for the hot path
- `bincode`: Deprecated in favor of `msgpack` for the server (per PR #2523); `postcard` is more compact
- Direct struct passing: Impossible due to Rust ABI instability

### Decision 3: Transforms in Runtime, Not in Plugins

Transforms (AddFields, DeleteFields, FilterFields, UpdateFields, ProtoConvert, FlatBufferConvert) run in the runtime process. This means:
- Plugins do not need to implement transform logic
- The runtime can observe and meter transform execution
- Transforms are composable in configuration without plugin awareness
- A single transform implementation serves all connectors

### Decision 4: State as Opaque Bytes

Source connector state (`ConnectorState`) is a `Vec<u8>` wrapper. The runtime does not interpret the state -- it simply persists and restores it. This was refined in PR #2378 (from structured to binary representation) to give connectors maximum flexibility in what they persist.

The PostgreSQL source, for example, serializes a `State` struct (containing `tracking_offsets` and `processed_rows`) via MessagePack. The Random source simply stores a `u64` as little-endian bytes. Both work through the same `ConnectorState` abstraction.

## 3.3 Bugs That Shaped the Architecture

| Bug | Impact | Architectural Change |
|---|---|---|
| [#2770](https://github.com/apache/iggy/issues/2770) FFI symbol collision with POSIX `open()` | Segfault when connectors access filesystem | All FFI functions prefixed with `iggy_sink_`/`iggy_source_` |
| [#2743](https://github.com/apache/iggy/issues/2743) State loss on source restart | Data re-processing after restart | State persistence moved earlier in the lifecycle; file handle kept open |
| [#2712](https://github.com/apache/iggy/issues/2712) Recursive plugin loading on Linux | Runtime crash from metadata reads | Harden plugin loading; only read TOML files from config directory |
| [#2928](https://github.com/apache/iggy/issues/2928) Auto-commit before sink processing | At-most-once instead of at-least-once delivery | Open issue -- `PollingMessages` auto-commit commits offsets before sink processing completes |
| [#2927](https://github.com/apache/iggy/issues/2927) `consume()` return value discarded | Silent sink failures | Open issue -- sink errors not propagated to runtime |

The two open issues (#2928 and #2927) represent the most significant delivery semantics gaps in the current architecture. They indicate the system is still maturing toward production-grade exactly-once or at-least-once guarantees.

---

# Key Finding 4: Competitive Positioning

## 4.1 Comparative Architecture Matrix

| Feature | **Iggy Connect** | **Kafka Connect** | **Pulsar IO** | **Redpanda Connect** |
|---|---|---|---|---|
| **Language** | Rust (native shared libraries) | Java (JVM classes) | Java (JVM) | Go (native binary) |
| **Plugin Model** | dlopen2 + C FFI | Java classloader | Java classloader | Compiled-in or WASM |
| **Isolation** | Separate process from server; plugins share runtime process | Same JVM as Connect worker | Same JVM as broker function workers | Single binary |
| **Serialization** | postcard (binary) across FFI | Java objects (no serialization boundary) | Java objects | Go interfaces (no serialization) |
| **Deployment** | Single binary + plugin .so files; Docker image | Connect cluster (distributed mode) or standalone | Part of Pulsar broker or standalone | Single static binary |
| **Config Format** | TOML files or HTTP API | JSON via REST API | YAML/JSON | YAML pipeline definitions |
| **Transforms** | SDK-provided (AddFields, DeleteFields, FilterFields, etc.) | SMTs (Single Message Transforms) | Pulsar Functions | Bloblang (custom DSL) |
| **Schema Support** | JSON, Raw, Text, Protobuf, FlatBuffers | Avro, JSON Schema, Protobuf (via Schema Registry) | Pulsar Schema (automatic) | Codec-based |
| **Connector Count** | ~12 native (7 sinks + 4 sources + HTTP sink) | Hundreds (Confluent Hub) | ~20+ built-in | 200+ (inputs + outputs + processors) |
| **Exactly-Once** | Not yet (open issues #2927, #2928) | Supported (with transactions) | Limited | At-least-once |
| **State Management** | File-based (per-connector state files) | Kafka internal topics | Pulsar internal topics | Stateless by design |
| **Metrics** | Prometheus + /stats endpoint | JMX + Prometheus (via Connect metrics) | Prometheus | Prometheus + OpenTelemetry |
| **Hot Reload** | Yes (PR #2781) | Yes (via REST API) | Yes (via admin API) | Yes (via API) |

## 4.2 Where Iggy Connect Excels

### Performance Ceiling

Iggy's Rust-native approach gives it a fundamental performance advantage. Where Kafka Connect requires JVM startup, GC tuning, and serialization through the Kafka protocol, Iggy connectors are compiled native code with:
- `mimalloc` as the global allocator (optimized for multi-threaded, small-allocation workloads)
- `simd_json` for JSON parsing (2-4x faster than standard JSON parsers)
- `postcard` for compact binary serialization (~10x smaller than JSON)
- Direct `IggyProducer`/`IggyConsumer` integration (no protocol translation layer)

### Memory Footprint

The README emphasizes "low memory footprint" as a first-class feature. A Kafka Connect worker typically requires 1-4 GB of JVM heap. The Iggy connector runtime, being a native Rust binary, can operate with an order of magnitude less memory.

### Developer Experience (for Rust developers)

The macro-based approach (`sink_connector!`, `source_connector!`) reduces the boilerplate of FFI plugin development to near-zero. A developer implements a 3-method trait and calls a macro. Compare this to Kafka Connect, where implementing a connector requires:
- `SourceConnector` / `SinkConnector` class
- `SourceTask` / `SinkTask` class
- Config definition class
- Schema specification
- Packaging as a JAR with plugin manifest

## 4.3 Where Iggy Connect Must Improve

### Delivery Semantics (Critical Gap)

The open issues #2927 (consume return value discarded) and #2928 (offset committed before processing) represent a **fundamental gap** in delivery guarantees. Without at-least-once semantics, the connector framework cannot be used in production scenarios requiring data integrity. Kafka Connect solves this with:
- Exactly-once support via Kafka transactions
- Configurable offset commit strategies
- Dead letter queues for failed records

Issue #2940 (partial write discussion) shows the team is aware of this gap and actively exploring solutions.

### Connector Ecosystem Scale

With ~12 native connectors vs. Kafka Connect's hundreds, the ecosystem is nascent. The [Connector Ecosystem Roadmap (Issue #2753)](https://github.com/apache/iggy/issues/2753) tracks 120+ target connectors across databases, cloud, IoT, and observability -- but execution will take significant community investment.

The presence of community PRs for ClickHouse (#2886), Delta Lake (#2889), and HTTP sink (#2925) suggests the framework's developer experience is attracting contributors. The InfluxDB connector (#2933) was a complete community contribution.

### Language Restriction

Connectors must be written in Rust (compiled as `cdylib`). The original Discussion #1670 envisioned language flexibility through IPC or iceoryx2, but the current FFI-based approach constrains the ecosystem to Rust developers. Redpanda Connect, by contrast, accepts plugins in any WASM-supported language. Kafka Connect leverages the vast Java ecosystem.

The Java connectors in the Iggy tree (Apache Pinot, Apache Flink) operate differently -- they use the Iggy Java SDK as regular clients, not as dynamically-loaded plugins. They are not part of the connector runtime.

### Partial Write Handling

Issue #2940 articulates an unsolved architectural question:

> "Some sinks can partially commit a batch in the external system and still return an error. That creates a hard tradeoff between not reporting full success when only part of the batch was written, not getting stuck replaying already-written messages forever, and keeping progress and restart behavior understandable."

The current `consume()` -> `i32` return type (0 for success, non-zero for failure) provides no mechanism for partial success reporting. A richer result type (e.g., `PartialResult { succeeded: usize, failed: Vec<FailedMessage> }`) would be needed.

## 4.4 Strategic Positioning Analysis

Iggy Connect occupies a distinct niche in the connector landscape:

```
                    High Performance
                         |
                    Iggy Connect
                         |
    Rust-only ----+------+------+---- Polyglot
                  |              |
                  |              Kafka Connect
                  |              Pulsar IO
                  |
             Redpanda Connect
                  |
                    Low Ecosystem
```

**The strategic bet is clear:** Iggy is building for a future where Rust becomes the dominant systems programming language for data infrastructure, and where connector performance matters as much as connector availability. The framework is designed to be the **fastest possible** connector runtime, accepting the tradeoff of a smaller ecosystem in exchange for performance characteristics that JVM-based alternatives cannot match.

The competitive moat, if one develops, will come from:
1. Connectors that *must* be fast (IoT, trading, real-time ML feature stores)
2. Connectors where memory footprint matters (edge deployments, embedded systems)
3. The Rust ecosystem's natural growth bringing more potential contributors

---

# Appendix A: Module Dependency Graph

```
core/connectors/
  |
  +-- sdk/                          (shared contract)
  |   +-- src/lib.rs                Source, Sink, Payload, Schema, Error
  |   +-- src/sink.rs               SinkContainer, sink_connector! macro
  |   +-- src/source.rs             SourceContainer, source_connector! macro
  |   +-- src/decoders/             JSON, Raw, Text, Proto, FlatBuffer decoders
  |   +-- src/encoders/             JSON, Raw, Text, Proto, FlatBuffer encoders
  |   +-- src/transforms/           AddFields, DeleteFields, FilterFields, UpdateFields, ProtoConvert, FlatBufferConvert
  |   +-- src/api.rs                ConnectorStatus, stats/info response types
  |   +-- src/retry.rs              Retry utilities
  |   +-- src/convert.rs            OwnedValue <-> serde_json conversion
  |   +-- src/log.rs                FFI-safe logging callback
  |
  +-- runtime/                      (orchestrator binary)
  |   +-- src/main.rs               Entry point, plugin loading, lifecycle management
  |   +-- src/sink.rs               Sink init, consumer setup, message processing loop
  |   +-- src/source.rs             Source init, producer setup, forwarding loop
  |   +-- src/transform.rs          Transform loading from config
  |   +-- src/state.rs              FileStateProvider for source connector state
  |   +-- src/context.rs            RuntimeContext (sinks, sources, metrics, config)
  |   +-- src/metrics.rs            Prometheus metrics (counters, gauges, families)
  |   +-- src/stats.rs              /stats endpoint data aggregation
  |   +-- src/stream.rs             Iggy client initialization
  |   +-- src/log.rs                Logging + OpenTelemetry setup
  |   +-- src/api/                  HTTP API (axum-based)
  |   +-- src/configs/              Config loading (local + HTTP providers)
  |   +-- src/manager/              Sink/Source manager (lifecycle, restart)
  |
  +-- sinks/
  |   +-- stdout_sink/              Debug output
  |   +-- quickwit_sink/            Quickwit search indexing
  |   +-- postgres_sink/            PostgreSQL writes
  |   +-- elasticsearch_sink/       Elasticsearch indexing
  |   +-- iceberg_sink/             Apache Iceberg (REST catalog, S3/GCS/Azure)
  |   +-- mongodb_sink/             MongoDB writes
  |   +-- influxdb_sink/            InfluxDB writes
  |   +-- http_sink/                Generic HTTP endpoint
  |
  +-- sources/
      +-- random_source/            Synthetic test data
      +-- postgres_source/          PostgreSQL (CDC + polling)
      +-- elasticsearch_source/     Elasticsearch polling
      +-- influxdb_source/          InfluxDB reads
```

# Appendix B: Key Commit Timeline

```
2025-05-28  ac44ad3b4  GENESIS: connectors runtime, SDK, 2 sinks, 1 source (PR #1826)
2025-06-06  ae1470cdf  Documentation pass
2025-06-17  8dc210aec  +Elasticsearch sink & source (PR #1872)
2025-06-17  827abedaa  +Protobuf support (PR #1886)
2025-09-01  775508467  +State storage for sources (PR #1948)
2025-09-01  00619ac25  +FlatBuffers support (PR #1957)
2025-09-01  a541c9b89  +PostgreSQL sink & source (PR #1959)
2025-09-23  0d3ca8e40  +Apache Iceberg sink (PR #2191)
2025-11-07  e0322b2bf  Split connector/runtime configs (PR #2317)
2025-11-07  5d73772e4  Configuration provider trait (PR #2345)
2025-12-03  734cb3430  +HTTP configuration provider (PR #2401)
2026-01-28  76e928a44  +Prometheus metrics & stats (PR #2633)
2026-02-05  0aa1f3864  Fix state/memory leaks (PR #2685)
2026-02-18  9ecbcc00d  Fix POSIX FFI collision (PR #2771)
2026-02-19  deb3eaad4  +Hot-reload connectors (PR #2781)
2026-02-25  f91486031  +MongoDB sink (PR #2815)
2026-03-12  c9e40b693  +HTTP sink (PR #2925)
2026-03-14  0460de9bd  +InfluxDB sink & source (PR #2933)
```

# Appendix C: Related Issues & Design Discussions

| Issue/Discussion | Status | Topic |
|---|---|---|
| [Discussion #1670](https://github.com/apache/iggy/discussions/1670) | Closed | Original connector architecture proposal |
| [#2753](https://github.com/apache/iggy/issues/2753) | Open | Connector ecosystem roadmap (120+ targets) |
| [#2940](https://github.com/apache/iggy/issues/2940) | Open | Partial write and replay-safe progress design |
| [#2928](https://github.com/apache/iggy/issues/2928) | Open | Auto-commit before sink processing (delivery gap) |
| [#2927](https://github.com/apache/iggy/issues/2927) | Open | consume() return value discarded |
| [#1846](https://github.com/apache/iggy/issues/1846) | Open | Avro payload support |
| [#2539](https://github.com/apache/iggy/issues/2539) | Open | ClickHouse sink connector |
| [#2956](https://github.com/apache/iggy/issues/2956) | Open | Amazon S3 sink connector |
| [#2500](https://github.com/apache/iggy/issues/2500) | Open | JDBC source and sink connectors |
| [#2540](https://github.com/apache/iggy/issues/2540) | Open | Redshift sink with S3 staging |

# Appendix D: Dependency Highlights

**SDK dependencies** (from `sdk/Cargo.toml`):
- `iggy` + `iggy_common`: Core Iggy types
- `postcard`: FFI boundary serialization
- `simd-json`: High-performance JSON parsing
- `prost` + `prost-types` + `protox`: Protocol Buffers support
- `flatbuffers`: FlatBuffers support
- `dashmap`: Concurrent HashMap for plugin instances
- `tokio`: Async runtime (each plugin gets its own)
- `tracing` + `tracing-subscriber`: Structured logging

**Runtime dependencies** (from `runtime/Cargo.toml`):
- `dlopen2`: Dynamic library loading
- `axum` + `tower-http`: HTTP API server
- `prometheus-client`: Prometheus metrics
- `mimalloc`: Memory allocator
- `figment` + `toml`: Configuration loading
- `opentelemetry` stack: Telemetry export
- `reqwest` + `reqwest-retry`: HTTP config provider
- `flume`: MPSC channel for source message forwarding
- `sysinfo`: System resource monitoring for /stats

---

## Confidence & Caveats

**Confidence Level: High** for architectural analysis and code structure (based on direct source code reading). **Medium** for competitive analysis (based on web research; competitive landscape evolves rapidly). **Medium-High** for evolution narrative (based on git history and issue tracker; some design discussions may have occurred outside GitHub).

**Areas requiring independent verification:**
- Performance claims (Iggy vs. Kafka Connect throughput) -- no benchmarks were found in the repository
- The exact memory footprint comparison between Iggy connectors and Kafka Connect workers
- Whether the WASM plugin path mentioned in Discussion #1670 is still under consideration

**Assumptions that could change the analysis:**
- If Iggy adds Kafka protocol compatibility (PR #3038 was a WIP attempt), the connector ecosystem argument changes dramatically
- If the delivery semantics issues (#2927, #2928) are not resolved, production adoption will be severely limited
- If Rust adoption in the data infrastructure community accelerates, the "Rust-only" constraint becomes less limiting over time

---

*Generated from source code analysis of `core/connectors/` at commit `411a697eb`, git history of 80+ commits, 50+ related issues, 47+ related PRs, and competitive landscape research.*

Sources:
- [Kafka Connect Architecture - Confluent Documentation](https://docs.confluent.io/platform/current/connect/design.html)
- [Kafka Connect for Confluent Platform](https://docs.confluent.io/platform/current/connect/index.html)
- [How to Use Pulsar IO Connectors](https://oneuptime.com/blog/post/2026-01-27-pulsar-io-connectors/view)
- [Kafka vs Pulsar - Confluent](https://www.confluent.io/kafka-vs-pulsar/)
- [Redpanda Connect - GitHub](https://github.com/redpanda-data/connect)
- [Redpanda Connect Documentation](https://docs.redpanda.com/redpanda-connect/components/connector-support-levels/)
- [Message Broker Comparison 2025 - Medium](https://medium.com/@BuildShift/kafka-is-old-redpanda-is-fast-pulsar-is-weird-nats-is-tiny-which-message-broker-should-you-32ce61d8aa9f)
- [Apache Iggy - Official Site](https://iggy.apache.org/)
- [Apache Iggy Connectors Introduction](https://iggy.apache.org/docs/connectors/introduction/)
- [Iggy Connectors Runtime Blog Post](https://iggy.apache.org/blogs/2025/06/06/connectors-runtime/)
- [Connectors Plugin Design Discussion #1670](https://github.com/apache/iggy/discussions/1670)
- [How to Build a Plugin System in Rust - Arroyo](https://www.arroyo.dev/blog/rust-plugin-systems/)
- [Dynamic Loading Plugins in Rust - NullDeref](https://nullderef.com/blog/plugin-dynload/)
- [Connector Ecosystem Roadmap - Issue #2753](https://github.com/apache/iggy/issues/2753)

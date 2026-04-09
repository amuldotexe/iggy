# Rough Codebase Map Summary

## Counts
- Code files: 82
- Symbols: 1271
- Import/include edges: 654
- Internal file edges: 0
- External reference edges: 654
- Graph edge cap used: 160

## Tooling
- rg: yes
- ctags: no
- ast-grep: yes
- dot: no

## Top Fan-Out Files
- runtime/src/main.rs: 25
- sinks/influxdb_sink/src/lib.rs: 23
- runtime/src/manager/source.rs: 22
- sources/influxdb_source/src/lib.rs: 22
- runtime/src/manager/sink.rs: 21
- sdk/src/transforms/proto_convert.rs: 19
- sdk/src/encoders/proto.rs: 18
- sinks/iceberg_sink/src/router/mod.rs: 18
- sinks/http_sink/src/lib.rs: 17
- runtime/src/api/mod.rs: 15

## Top Fan-In Files
- none

## Pointer-First Retrieval Pattern
Use `symbols.tsv` and `internal_file_edges.tsv` first. Read code spans only when needed via `file:start:end`.

# MongoDB Source Connector

Polls a MongoDB collection and streams documents to Iggy topics.

## Try It

Insert a document into MongoDB and see it arrive as an Iggy message.

**Prerequisites**: Docker running, project built (`cargo build` from repo root).

```bash
# Start MongoDB
docker run -d --name mongo-test -p 27017:27017 mongo:7

# Start iggy-server (terminal 2)
IGGY_ROOT_USERNAME=iggy IGGY_ROOT_PASSWORD=iggy ./target/debug/iggy-server

# Create stream and topic
./target/debug/iggy -u iggy -p iggy stream create demo_stream
./target/debug/iggy -u iggy -p iggy topic create demo_stream demo_topic 1

# Setup connector config
mkdir -p /tmp/mdb-source-test/connectors
cat > /tmp/mdb-source-test/config.toml << 'TOML'
[iggy]
address = "localhost:8090"
username = "iggy"
password = "iggy"
[state]
path = "/tmp/mdb-source-test/state"
[connectors]
config_type = "local"
config_dir = "/tmp/mdb-source-test/connectors"
TOML
cat > /tmp/mdb-source-test/connectors/source.toml << 'TOML'
type = "source"
key = "mongodb"
enabled = true
version = 0
name = "test"
path = "target/debug/libiggy_connector_mongodb_source"
[[streams]]
stream = "demo_stream"
topic = "demo_topic"
schema = "json"
batch_length = 100
[plugin_config]
connection_uri = "mongodb://localhost:27017"
database = "test_db"
collection = "events"
poll_interval = "1s"
payload_format = "json"
TOML

# Start connector (terminal 3)
IGGY_CONNECTORS_CONFIG_PATH=/tmp/mdb-source-test/config.toml ./target/debug/iggy-connectors

# Insert a document into MongoDB
docker exec mongo-test mongosh --quiet --eval \
  'db.getSiblingDB("test_db").events.insertOne({"hello":"iggy","ts":new Date()})'

# Wait for poll cycle, then check Iggy
sleep 2
./target/debug/iggy -u iggy -p iggy message poll --offset 0 -m 10 demo_stream demo_topic 1
```

Expected: the inserted document appears as an Iggy message payload.

Cleanup: `docker rm -f mongo-test && rm -rf /tmp/mdb-source-test`

## Quick Start

```toml
[[streams]]
stream = "events"
topic = "raw_events"
schema = "json"
batch_length = 100

[plugin_config]
connection_uri = "mongodb://localhost:27017"
database = "mydb"
collection = "events"
poll_interval = "5s"
```

## Configuration

| Option | Default | Description |
| ------ | ------- | ----------- |
| `connection_uri` | **required** | MongoDB URI |
| `database` | **required** | Database name |
| `collection` | **required** | Collection name |
| `poll_interval` | `10s` | Polling frequency |
| `batch_size` | `1000` | Max documents per poll |
| `tracking_field` | `_id` | Field for incremental `$gt` filter |
| `initial_offset` | none | Starting value (ignored if state exists) |
| `query_filter` | none | Additional MongoDB filter (JSON string) |
| `projection` | none | Field projection (JSON string) |
| `payload_format` | `json` | `json`, `bson`, or `string` |
| `payload_field` | none | Extract a single field instead of entire document |
| `snake_case_fields` | `false` | Convert camelCase keys to snake_case |
| `include_metadata` | `false` | Inject source collection and poll timestamp |
| `delete_after_read` | `false` | Delete documents after processing |
| `processed_field` | none | Boolean field to mark instead of deleting |
| `max_pool_size` | driver default | Connection pool size |
| `verbose_logging` | `false` | Log at info instead of debug |
| `max_retries` | `3` | Retry attempts for transient errors |
| `retry_delay` | `1s` | Base delay (`retry_delay * attempt`) |

## Tracking Field Types

The connector currently supports these BSON kinds for `tracking_field` values:

- `Int32` and `Int64`
- `Double`
- `String`
- `ObjectId`
- `DateTime`

Notes:

- `Int32` values are normalized into typed `Int64` state when persisted.
- Fresh checkpoints are saved with type metadata, so typed offsets round-trip without coercion.
- Unsupported tracking kinds fail the poll with an explicit error naming the collection, tracking field, observed BSON kind, and supported kinds.
- Custom ObjectId fields that are not named `_id` are still compared as strings.

### Legacy State Compatibility

Older persisted state may still contain untyped string offsets.

That legacy fallback remains supported for backward compatibility, but it is less precise than the typed state path:

- numeric-looking strings such as `"42"` may be interpreted as numeric BSON
- 24-character hex strings on `_id` may be interpreted as `ObjectId`

Once the connector saves a fresh typed checkpoint, that ambiguity goes away for future resumes.

## Testing

Requires Docker. Testcontainers starts MongoDB 7 + iggy-server automatically.

```bash
cargo test --test mod -- mongodb_source
```

This runs 7 E2E tests against a real MongoDB instance:

- `source_polls_documents_to_iggy` — documents polled and delivered as Iggy messages
- `delete_after_read_removes_documents` — processed documents deleted from collection
- `mark_processed_sets_field` — processed documents marked with boolean field
- `state_persists_across_connector_restart` — offset survives connector restart
- `source_polls_documents_by_object_id` — ObjectId `_id` tracking (default)
- `source_delete_after_read_with_object_id` — delete mode with ObjectId tracking
- `source_mark_processed_with_object_id` — mark mode with ObjectId tracking

Unit tests (no Docker):

```bash
cargo test -p iggy_connector_mongodb_source
```

All 11 E2E tests (4 sink + 7 source) in one command:

```bash
cargo test --test mod -- mongodb
```

## Delivery Semantics

This connector provides **at-least-once** delivery semantics.

### Behavior

- Messages may be delivered more than once on retry or restart
- The source stages each polled batch and only commits progress after the runtime successfully sends the batch to Iggy
- Checkpoint persistence and `delete_after_read` / `processed_field` side effects run in the post-send commit path
- If the downstream send or post-send mark/delete fails, the same documents will be re-polled and may be delivered again
- If delete/mark side effects touch fewer documents than expected but MongoDB still reports success, the connector only warns and continues; it does not roll back delivery or reconcile the side effect automatically

### Known Limitations

- Custom ObjectId fields (not named `_id`) use string comparison
- Legacy untyped string-state fallback may reinterpret numeric-looking strings or `_id`-like strings until a typed checkpoint is saved
- Non-unique tracking fields can stall progress at batch boundaries; use a unique tracking field or lower `batch_size`

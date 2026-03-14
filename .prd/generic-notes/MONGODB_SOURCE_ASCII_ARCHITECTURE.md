# MongoDB Source Connector - ASCII Architecture Overview

## 1. Core Data Flow (The Main Loop)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MONGODB SOURCE CONNECTOR                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐               │
│   │   MongoDB    │     │   Connector  │     │    Iggy      │               │
│   │  Collection  │     │    State     │     │   Topics     │               │
│   └──────┬───────┘     └──────┬───────┘     └──────┬───────┘               │
│          │                    │                     │                        │
│          │  1. POLL           │                     │                        │
│          │◄───────────────────┤                     │                        │
│          │                    │                     │                        │
│          │  2. Documents      │                     │                        │
│          │  (batch + 1)       │                     │                        │
│          ├───────────────────►│                     │                        │
│          │                    │                     │                        │
│          │                    │  3. Convert to      │                        │
│          │                    │     Messages        │                        │
│          │                    │     + Store PENDING │                        │
│          │                    │                     │                        │
│          │                    │  4. SEND            │                        │
│          │                    ├────────────────────►│                        │
│          │                    │                     │                        │
│          │                    │  5. ACK (success)   │                        │
│          │                    │◄────────────────────┤                        │
│          │                    │                     │                        │
│          │  6. Delete/Mark    │                     │                        │
│          │◄───────────────────┤  (if configured)    │                        │
│          │                    │                     │                        │
│          │                    │  7. COMMIT          │                        │
│          │                    │  PENDING → COMMITTED│                        │
│          │                    │                     │                        │
└──────────┴────────────────────┴─────────────────────┴────────────────────────┘
```

## 2. State Machine (Rust `enum` concept explained)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            SOURCE STATE                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   In Rust, we track state with a struct containing two parts:               │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  SourceState {                                                       │   │
│   │      committed_state: State,     // ✅ SAVED - won't be resent      │   │
│   │      pending_batch: Option<...>, // ⏳ IN-FLIGHT - may be resent    │   │
│   │  }                                                                   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   State Transitions:                                                         │
│                                                                              │
│   ┌─────────────┐   poll()    ┌─────────────┐   send OK   ┌─────────────┐  │
│   │   EMPTY     │────────────►│   PENDING   │────────────►│  COMMITTED  │  │
│   │             │             │  batch=3    │             │  offset=42  │  │
│   └─────────────┘             └──────┬──────┘             └─────────────┘  │
│                                      │                                       │
│                                      │ send FAIL                             │
│                                      ▼                                       │
│                               ┌─────────────┐                               │
│                               │  DISCARDED  │  ← discard_polled_messages() │
│                               │  (retry)    │    Next poll gets same data  │
│                               └─────────────┘                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 3. Batch + Extra Document Pattern (Gap Detection)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    DUPLICATE BOUNDARY DETECTION                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Why fetch batch_size + 1? To detect if we're at a "duplicate boundary"    │
│                                                                              │
│   Example: batch_size = 3, tracking_field = "seq"                           │
│                                                                              │
│   ┌───────────────────────────────────────────────────────────────────────┐ │
│   │  MongoDB Documents (sorted by seq):                                    │ │
│   │                                                                        │ │
│   │   seq: 1 ──┐                                                          │ │
│   │   seq: 2   │  ◄── BATCH (3 docs, sent to Iggy)                        │ │
│   │   seq: 3 ──┘                                                          │ │
│   │   seq: 3   ◄── EXTRA doc (used for boundary check)                    │ │
│   │                                                                        │ │
│   └───────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│   Decision Tree:                                                             │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │   Is extra_offset == batch_last_offset?                              │   │
│   │                                                                       │   │
│   │       ┌──────YES──────┐                                              │   │
│   │       │               │                                              │   │
│   │       ▼               ▼                                              │   │
│   │   ┌───────┐       ┌───────┐                                          │   │
│   │   │ _id   │       │ non   │                                          │   │
│   │   │ field │       │ _id   │                                          │   │
│   │   └───┬───┘       └───┬───┘                                          │   │
│   │       │               │                                               │   │
│   │       ▼               ▼                                               │   │
│   │   OK!              Is there a                                         │   │
│   │   (_id is          previous distinct?                                │   │
│   │   always unique)       │                                              │   │
│   │                    YES │   NO                                         │   │
│   │                       ▼    ▼                                          │   │
│   │                    Roll  FAIL!                                        │   │
│   │                    back  (all equal,                                  │   │
│   │                    to    can't progress)                              │   │
│   │                    prev                                              │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ASCII of checkpoint decision:                                              │
│                                                                              │
│   Batch:  [seq=1] [seq=2] [seq=3] [seq=3(EXTRA)]                            │
│                          ▲         ▲                                        │
│                          │         └── extra_offset = 3                     │
│                          └── batch_max = 3                                  │
│                                                                              │
│   Since extra == batch_max AND tracking != "_id":                           │
│   → Checkpoint rolls back to previous DISTINCT (seq=2)                      │
│   → This ensures seq=3 duplicates are re-fetched next poll                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 4. Typed Offset Tracking (Rust Enums)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TRACKING OFFSET TYPES                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Rust uses enums to ensure type safety. Think of them as "tagged unions":  │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  enum TrackingOffsetValue {      // NEW (typed)                      │   │
│   │      Int64(i64),                 // e.g., 42                         │   │
│   │      Double(f64),                // e.g., 3.14                       │   │
│   │      String(String),             // e.g., "user_123"                 │   │
│   │      ObjectIdHex(String),        // e.g., "507f1f77bcf86cd799439011" │   │
│   │      DateTimeMillis(i64),        // e.g., 1678886400000              │   │
│   │  }                                                                   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  enum PersistedTrackingOffset {  // Wrapper for compatibility        │   │
│   │      Typed(TrackingOffsetValue), // New format (preserves type)      │   │
│   │      LegacyString(String),       // Old format (backward compat)     │   │
│   │  }                                                                   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   Why this matters:                                                          │
│                                                                              │
│   ┌──────────────────────────────────────────────────────────────────┐     │
│   │                                                                   │     │
│   │   LEGACY: State saved as {"offset": "42"}                        │     │
│   │           → Is "42" a number or the string "42"?                 │     │
│   │           → MongoDB query needs EXACT type match!                │     │
│   │                                                                   │     │
│   │   TYPED: State saved as {"type":"int64","value":42}              │     │
│   │           → Query uses Bson::Int64(42) ✓                         │     │
│   │                                                                   │     │
│   └──────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│   ASCII of the problem:                                                      │
│                                                                              │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │  MongoDB:  { seq: 42 }     // seq is a NUMBER                     │    │
│   │                                                                    │    │
│   │  WRONG query:  { seq: { $gt: "42" } }  // string "42"             │    │
│   │  → Returns NOTHING (type mismatch)                                │    │
│   │                                                                    │    │
│   │  CORRECT query: { seq: { $gt: 42 } }   // number 42               │    │
│   │  → Returns documents with seq > 42                                │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 5. Filter Composition (Query Building)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    FILTER BUILDING                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   The connector builds a MongoDB query from multiple sources:               │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                                                                      │   │
│   │   1. Tracking clause:  { tracking_field: { $gt: last_offset } }     │   │
│   │      └── Auto-generated from checkpoint state                        │   │
│   │                                                                      │   │
│   │   2. User query_filter (optional):                                   │   │
│   │      └── e.g., { "tenant": "alpha", "status": "active" }            │   │
│   │                                                                      │   │
│   │   3. Processed field (optional):                                     │   │
│   │      └── e.g., { "is_processed": false }                            │   │
│   │                                                                      │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   Combined with $and:                                                        │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  {                                                                    │   │
│   │    "$and": [                                                          │   │
│   │      { "seq": { "$gt": 42 } },              // tracking              │   │
│   │      { "tenant": "alpha" },                 // user filter           │   │
│   │      { "is_processed": false }              // processed field       │   │
│   │    ]                                                                  │   │
│   │  }                                                                    │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   Key insight: User filter CANNOT overwrite the tracking clause!            │
│   Even if user puts { "seq": { "$lt": 100 } } in query_filter,             │
│   the $gt:42 tracking clause is preserved via $and composition.            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 6. Side Effects (Delete After Read / Mark Processed)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SIDE EFFECTS TIMING                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   CRITICAL: Side effects happen ONLY after successful send to Iggy!         │
│                                                                              │
│   Timeline:                                                                  │
│                                                                              │
│   ┌────────┬─────────────────────────────────────────────────────────────┐  │
│   │  TIME  │  ACTION                                                      │  │
│   ├────────┼─────────────────────────────────────────────────────────────┤  │
│   │   T1   │  poll() → get batch from MongoDB                            │  │
│   │   T2   │  Convert to messages, store as PENDING                      │  │
│   │   T3   │  Send messages to Iggy                                      │  │
│   │   T4   │  ◄── SEND FAILS ──► discard_polled_messages_now()           │  │
│   │        │                    (MongoDB unchanged, will retry)          │  │
│   │        │                                                              │  │
│   │   T4'  │  ◄── SEND SUCCEEDS ──► commit_polled_messages_now()         │  │
│   │   T5   │  Execute side effects (delete/mark)                         │  │
│   │   T6   │  Move PENDING → COMMITTED state                             │  │
│   └────────┴─────────────────────────────────────────────────────────────┘  │
│                                                                              │
│   This ensures at-least-once delivery:                                      │
│   - If Iggy doesn't get the message, MongoDB keeps the document            │
│   - If Iggy gets the message, we can safely delete/mark                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 7. Error Handling Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ERROR HANDLING                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                                                                      │   │
│   │   TRANSIENT ERRORS (retry with backoff):                             │   │
│   │   ├── timeout                                                        │   │
│   │   ├── network failure                                                │   │
│   │   ├── connection refused                                             │   │
│   │   └── pool exhausted                                                 │   │
│   │                                                                      │   │
│   │   PERMANENT ERRORS (fail immediately):                               │   │
│   │   ├── unsupported BSON kind in tracking field                        │   │
│   │   ├── missing tracking field                                         │   │
│   │   ├── all-equal duplicate boundary (can't progress)                  │   │
│   │   └── invalid configuration                                          │   │
│   │                                                                      │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   Retry Logic:                                                               │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │   for attempt in 1..max_retries:                                     │   │
│   │       match execute_poll():                                          │   │
│   │           Ok(batch) => return batch                                  │   │
│   │           Err(e) if is_transient(e) && attempt < max_retries:        │   │
│   │               sleep(retry_delay * attempt)  // exponential backoff   │   │
│   │               continue                                               │   │
│   │           Err(e) => return Err(e)  // give up                        │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Summary for Rust Beginners

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    KEY RUST CONCEPTS USED                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. enum = A type that can be ONE of several variants                      │
│      - TrackingOffsetValue::Int64(42)                                       │
│      - TrackingOffsetValue::String("abc".to_string())                       │
│                                                                              │
│   2. Option<T> = Maybe has a value, maybe not                               │
│      - Some(value) → has value                                              │
│      - None → no value                                                      │
│                                                                              │
│   3. Result<T, E> = Operation that might fail                               │
│      - Ok(value) → success                                                  │
│      - Err(error) → failure                                                 │
│                                                                              │
│   4. async/await = Asynchronous programming                                 │
│      - async fn poll() → returns a Future                                   │
│      - .await → wait for the Future to complete                             │
│                                                                              │
│   5. Mutex<T> = Thread-safe mutable state                                   │
│      - .lock().await → get exclusive access                                 │
│      - Rust ensures no data races at compile time!                          │
│                                                                              │
│   6. impl Trait for Struct = Implement interface                            │
│      - impl Source for MongoDbSource { ... }                                │
│      - Means MongoDbSource can be used anywhere Source is expected          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

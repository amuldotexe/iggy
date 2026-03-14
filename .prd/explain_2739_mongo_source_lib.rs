/* 
================================================================================
    M O N G O D B   →   I G G Y   S O U R C E   C O N N E C T O R   (E L I 5)
================================================================================

This file is a **story mode** version of the real connector:

    real code:     `core/connectors/sources/mongodb_source/src/lib.rs`
    this file:     `iggy/.prd/explain_2739_mongo_source_lib.rs`

It keeps the same logic but adds a lot of ASCII-art + comments explaining:

  - what each external crate is for
  - what the main types & functions do
  - why certain design choices were made

Think of it as:

      ┌──────────────────────────────────────────────────────────┐
      │                 MongoDB collection                       │
      │  (documents live here and keep getting inserted)        │
      └───────────────┬─────────────────────────────────────────┘
                      │
                      │ (poll in batches)
                      ▼
      ┌──────────────────────────────────────────────────────────┐
      │                MongoDbSource (this code)                │
      │  - connects to MongoDB                                 │
      │  - remembers where it left off (offset)                │
      │  - turns docs into Iggy messages                       │
      └───────────────┬─────────────────────────────────────────┘
                      │
                      │ (ProducedMessages)
                      ▼
      ┌──────────────────────────────────────────────────────────┐
      │                      Iggy core                          │
      │  - takes messages from many sources                    │
      │  - routes / persists / processes them                  │
      └─────────────────────────────────────────────────────────┘

Everything else in this file supports that flow.

================================================================================
EXTERNAL CRATES USED (ELI5 + WHY)
================================================================================

  async_trait
  ───────────
  - Lets us write async trait methods (`async fn` inside a trait impl).
  - Rust traits don’t directly allow async methods without some macro help.

  futures::TryStreamExt
  ─────────────────────
  - Adds handy methods like `.try_collect()` to async streams, so we can
    pull all documents out of a MongoDB cursor with one call.
================================================================================
WHAT IS "ASYNC"? (THE CONCEPT IN PLAIN ENGLISH)
================================================================================

Imagine a restaurant waiter (your code) and a chef (MongoDB server, disk, etc.).

BLOCKING WORLD (one thread, one task):

      Waiter takes order from Table A → walks to kitchen → waits there (blocks) →
      chef finishes → waiter walks back to Table A → delivers → only then
      can take order from Table B.

Problem: If kitchen is slow for Table A, waiter **cannot help Table B at all**.
      The whole thread is "stuck" waiting.

ASYNC WORLD (many small tasks, pause/resume):

      Waiter takes order from Table A → goes to kitchen → **"pause, go help others!"**
      Takes order from Table B → goes to kitchen → **"pause, go help others!"**
      Chef finishes A → Waiter sees "Table A ready!" → **resumes and delivers A**
      Chef finishes B → Waiter sees "Table B ready!" → **resumes and delivers B**

Key idea: instead of "I sit here until done", it's "I note where I paused,
      go do other work, then resume here when data is ready."

ASCII diagram:

      BLOCKING                              ASYNC (pause/resume)
      ────────┐                          ───────────────┐
      Table A    │                          Table A         │
         │       │  ──wait───►厨师            │  ──wait───►厨师
         ▼       │            │               │         │            │
      (blocked)  │            ▼ (done)       │         ▼ (done)    │
                 │         (continue)       │        (continue)   │
      Table B    │                          Table B         │
         ▼                              ▼               │
      (blocked)  ◄─── waiter returns ────────────┘           ◄─── waiter returns ─────────

In async, one waiter can juggle many tables without waiting fully for one.

================================================================================
HOW DOES ASYNC WORK IN RUST? (THE MECHANICS)
================================================================================

Three pieces: async fn, Future, await.

async fn (what you WRITE):

      async fn send_to_mongo(data: String) -> Result<(), Error> {
          // I might do slow work, network calls, disk I/O...
      }

What it REALLY means:

      "This function does NOT run to completion right here.
       Instead, it returns a FUTURE object (like a ticket)."

ASCII:

      ┌────────────────────────────────────────────┐
      │  async fn send_to_mongo(...) -> Future<...> │
      │  ┌─────────────────────────────────────────┐  │
      │  │ Returns immediately, NOT by running! │  │
      │  │ Think of it as: "I'll start this │  │
      │  │  whenever someone asks me to move." │  │
      │  └─────────────────────────────────────────┘  │
      └────────────────────────────────────────────────────┘

Future (the ticket):

      - It has a "current status": not ready, pending, ready.
      - Runtime (Tokio) can check many Futures and wake up the ones
        that become ready (Mongo responded, timer fired, etc.).

await (what you USE):

      async fn main_logic() -> Result<(), Error> {
          let future = send_to_mongo("data".to_string());
          // This is a "pause here" point
          let result = future.await?;
          // Code here resumes ONLY after future completes
      }

ASCII diagram:

      Timeline:
      t0     t1           t2           t3          t4          t5
      │       │            │            │           │           │
      ▼       │            │            │           │           ▼
    Start ──► Pause ────────► Resume ──────► Pause ────► Resume ──► Done
           (await)         (after)       (await)     (after)      (await)

Every `await` is a potential handoff point: "if runtime wants to run something else,
      it can pause me here."

Why this matters for this connector:

      ┌────────────────────────────────────────────────────┐
      │  MongoDB (slow, over network)                │
      │        ◄─── network I/O, takes time ────► │
      └────────────────────────────────────────────────────┘
                      │
                      │ (await) ── can pull over to work on other connectors
                      ▼
      ┌────────────────────────────────────────────────────┐
      │  MongoDbSource (async)                    │
      │  - while waiting on Mongo, Tokio can       │
      │    schedule other tasks on same thread       │
      └────────────────────────────────────────────────────┘

================================================================================
WHY DOESN'T RUST HAVE "ASYNC" AS A LANGUAGE FEATURE?
================================================================================

Imagine Rust evolution:

      Year 2010 (early):   blocking, no async, one task per thread
      Year 2015 (middle): add Futures, but still explicit types
      Year 2020+ (now):     people want "async fn" in traits, but traits have a problem

THE TRAIT PROBLEM:

Traits say: "If you implement these methods, I can use you as a Source."

But a trait method is a CONTRACT: it must return ONE concrete type, e.g.:

      trait Source {
          fn open(&mut self) -> Result<(), Error>;  // ← ONE type, clear
      }

If we allow `async fn`, what does it return?

      trait Source {
          async fn open(&mut self) -> Result<(), Error>;  // ← WHAT is the return type?
      }

Each async fn wants to return its OWN unique Future type. Traits can't say "any Future that
eventually yields this."

ASCII:

      Trait definition:
      ┌────────────────────────────────────────────┐
      │  trait Source {                       │
      │      fn open(...) -> Result<_, Error>; │
      │  }                                  │
      │  Problem: ONE return type only!       │
      └────────────────────────────────────────────┘
      If we try async fn:
      ┌────────────────────────────────────────────┐
      │  trait Source {                       │
      │      async fn open(...)               │
      │          -> ????Future???            │  // ← Impossible: each async fn
      │  }                                  │    creates its own Future type!
      └────────────────────────────────────────────┘
      Solution: async_trait macro transforms it:
      ┌────────────────────────────────────────────┐
      │  What you write:                      │
      │      async fn open(...) { ... }        │
      └────────────────────────────────────────────┘
                       │
                       │ (macro rewrites)
                       ▼
      ┌────────────────────────────────────────────┐
      │  What compiler sees:                   │
      │      fn open(...) ->                   │
      │          Box<dyn Future + Send + ...>;   │  // ← Single concrete type!
      └────────────────────────────────────────────┘

So traits can promise ONE type, which is a boxed Future (wrapper that can hold any
specific Future).

================================================================================
5-YEAR-OLD'S MINDSET EXPLAINED (ASYNC EVOLUTION)
================================================================================

Async/await didn't exist in one day. It evolved through several generations:

GENERATION 1: CALLBACKS (Year ~2010)

      do_mongo_operation(data, (error, result) => {
          if (error) { handle_error(error); }
          else { handle_result(result); }
      });

      // Callback hell: nested, hard to read, hard to reason about flow.

ASCII:

      Code flow with callbacks:
      start ──► call_with_callback ──► ??? who knows when it comes back?
                                    │                    │
                                    ◄────────────────────┘
      (control inverted; callback somewhere in future)

GENERATION 2: PROMISES (Year ~2013)

      let promise = mongo_operation(data);
      promise.then(result => handle_result(result))
            .catch(error => handle_error(error));

      // Better: you get a "ticket" (promise) that represents "future result".
      // Chain them: promise.then(...).then(...).

ASCII:

      Promise chain:
      operation ──► Promise ──► .then() ──► .then() ──► .catch()

GENERATION 3: ASYNC/AWAIT (NOW, ~2016+)

      async fn main() -> Result<(), Error> {
          let result = await mongo_operation(data)?;
          handle_result(result);
          let next_result = await another_operation()?;
      }

      // Looks "blocking" code! But it's async under the hood.
      // Easier to read: control flows DOWN the page, not jumping to callbacks.

ASCII:

      async/await code flow:
      step1 ──► await ──► step2 ──► await ──► step3
                            │                   ▼ (pause)
                            │                   ──► step4
                            │                   (resume)
                            ◄───────────────────┘

Why this evolved:

      1. Callbacks: Powerful, but messy (inverted version of control).
      2. Promises: Better chainability, but still "callback-ish".
      3. async/await: Looks blocking, but async. Best of both worlds.

How Rust's async_trait fits this evolution:

      - `Future` is like a Promise (represents "work that will complete").
      - `await` is like `.then()` (pause until it's done).
      - `async_trait` is the glue that lets traits use `async fn` like modern
        JavaScript/Python/C# use async/await.

In this file's context:

      ┌────────────────────────────────────────────┐
      │  Source trait (Iggy SDK)              │
      │  - wants connectors that can async     │
      │  poll without blocking                  │
      └────────────────────────────────────────────┘
              │
              │ Uses async_trait macro
              ▼
      ┌────────────────────────────────────────────┐
      │  MongoDbSource implementation          │
      │  - async fn open, poll, etc.       │
      │  - Returns Futures instead of blocking     │
      │  - Tokio drives them forward           │
      └────────────────────────────────────────────┘

Result: Iggy can run MANY MongoDB sources in parallel on few threads because
      each `await` point is a chance for Tokio to switch to other work.

  humantime::Duration as HumanDuration
  ────────────────────────────────────
  - Lets config use human strings like "10s" instead of only raw numbers.
  - Example: poll_interval "10s" → `Duration::from_secs(10)`.

  iggy_common::{DateTime, Utc}
  ────────────────────────────
  - Shared datetime types in the Iggy ecosystem.
  - We use them to track last poll times, etc.

  iggy_connector_sdk::{ConnectorState, Error, ProducedMessage, ProducedMessages,
                       Schema, Source, source_connector}
  ──────────────────────────────────────────────────────────────────────────────
  - The SDK that defines **what a connector must look like**.
  - `Source` trait = “thing that can produce messages”.
  - `ProducedMessage` / `ProducedMessages` = what we output.
  - `ConnectorState` = how we save/restore our progress (offsets, counters).
  - `Error` = common error type Iggy expects from connectors.
  - `Schema` = tells Iggy how to interpret our payload (JSON, raw bytes, text).
  - `source_connector!` = macro that wires this struct into the SDK registry.

  mongodb::{Client, Collection, bson, options}
  ─────────────────────────────────────────────
  - Official MongoDB Rust driver.
  - `Client` = connection to server.
  - `Collection<Document>` = handle for a particular collection.
  - `bson::{Bson, Document, doc, oid::ObjectId}` = BSON value & document types.
  - `options::{ClientOptions, FindOptions}` = configuration for client & queries.

  serde::{Deserialize, Serialize}
  ───────────────────────────────
  - De/serialization library.
  - Used for:
      - connector config (reading from config)
      - persisted state (saving offsets, etc.)

  std::collections::HashMap
  ─────────────────────────
  - Key-value store.
  - Here: tracks offsets per collection: `collection_name -> offset`.

  std::str::FromStr
  ─────────────────
  - Trait that lets us parse from `&str` into other types.
  - Needed by `HumanDuration::from_str`.

  std::time::Duration
  ───────────────────
  - Standard duration type for sleeps and retry intervals.

  tokio::sync::Mutex
  ──────────────────
  - Async mutex protecting the connector’s internal state.
  - We poll / commit from async contexts; this keeps shared state safe.

  tracing::{debug, info, warn}
  ────────────────────────────
  - Structured logging.
  - We use different levels:
      - info: lifecycle events (open/close, success)
      - debug: detailed but less noisy messages
      - warn: odd or risky conditions (duplicate offsets, partial deletes)

  uuid::Uuid
  ──────────
  - Generates unique message IDs for `ProducedMessage`.

================================================================================
WHAT IS MONGODB / A COLLECTION? (PYRAMID VIEW)
================================================================================

At a very zoomed-out level, MongoDB is:

      ┌──────────────────────────────────────────────┐
      │                MongoDB server               │
      │  - runs as a process somewhere             │
      │  - holds many logical databases            │
      └───────────────────────────┬────────────────┘
                                  │
                                  ▼
      ┌──────────────────────────────────────────────┐
      │              MongoDB database                │
      │  - like a “namespace” for related data       │
      │  - e.g.   database = "analytics"            │
      └───────────────────────────┬────────────────┘
                                  │
                                  ▼
      ┌──────────────────────────────────────────────┐
      │             MongoDB collection               │
      │  - like a table, but for JSON-ish docs      │
      │  - e.g.   collection = "events"             │
      └───────────────────────────┬────────────────┘
                                  │
                                  ▼
      ┌──────────────────────────────────────────────┐
      │               MongoDB document               │
      │  - one record, stored as BSON               │
      │  - key/value fields, flexible schema        │
      │  - always has an `_id` field (unique)       │
      └──────────────────────────────────────────────┘

In **this** code:

- `MongoDbSourceConfig.database` chooses which **database** to read.
- `MongoDbSourceConfig.collection` chooses which **collection** inside that DB.
- Each call to `poll()`:
  - asks MongoDB: “give me the next N documents from this collection
    whose tracking field is greater than the last offset I saw”.
  - wraps those documents as Iggy `ProducedMessage`s so the rest of Iggy
    can process them like a stream.

Why we need a collection here:

- The connector is **not** a generic “Mongo sniff everything” tool.
- It is focused on **one specific stream of documents**, defined by:
  - which database
  - which collection
  - which tracking field (e.g. `_id` or `seq`)
  - and optional extra filters (tenant, kind, processed flags, etc.).
- That combination is what turns Mongo into a **log-like stream** that Iggy can
  pull from in order, without re-reading the same documents forever.

You can think of the collection as the “inbox” that this connector is reading:

      ┌──────────────────────────────────────────────┐
      │         MongoDB collection = INBOX          │
      │  - producers insert new documents           │
      └───────────────────────────┬────────────────┘
                                  │
                                  │ (MongoDbSource polls here)
                                  ▼
      ┌──────────────────────────────────────────────┐
      │            MongoDbSource (this crate)        │
      │  - remembers last offset per collection      │
      │  - fetches newer docs only                   │
      │  - emits ProducedMessages to Iggy            │
      └──────────────────────────────────────────────┘

================================================================================
DESIGN OVERVIEW (STATE & FLOW)
================================================================================

Main idea:

  - `MongoDbSourceConfig`   = what user sets in config file.
  - `State`                 = what we remember across restarts (offsets, counts).
  - `SourceState`           = wrapper around `State` + currently pending batch.
  - `MongoDbSource` struct  = holds:
        * id, Mongo client, config
        * state protected by async Mutex
        * timing & retry settings

And the lifecycle roughly goes:

  1. `new(...)`      → read config & optional saved state, set defaults.
  2. `open()`        → build Mongo client, ping server, warn if collection missing.
  3. `poll()`        → every poll interval:
                         - query Mongo for new docs
                         - convert to `ProducedMessage`s
                         - stash checkpoint info in `pending_batch`
  4. `commit_polled_messages_now()` or
     `discard_polled_messages_now()` → decide what to do with that pending batch:
                         - commit: update offsets, maybe delete or mark processed.
                         - discard: drop pending batch, don’t advance offsets.
  5. `close()`       → drop client, log final stats.

ASCII picture of state:

      ┌───────────────────┐
      │  MongoDbSource    │
      │───────────────────│
      │ id                │
      │ client (Option)   │
      │ config            │
      │ retry_delay       │
      │ poll_interval     │
      │ state (Mutex)     │───>  ┌────────────────────┐
      └───────────────────┘      │   SourceState      │
                                 │--------------------│
                                 │ committed_state    │───>  last_poll_time
                                 │                    │      tracking_offsets
                                 │                    │      processed_documents
                                 │ pending_batch?     │
                                 └────────────────────┘

The “tracking offset” is like a **bookmark** into your collection so we don’t
re-read the same documents forever.

================================================================================
END OF BIG PICTURE. BELOW IS THE REAL CODE + INLINE EXPLANATIONS.
================================================================================
*/

/* Original Apache 2.0 license from the source file.
 * We keep it unchanged because this file is derived from that code.
 */
/* Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Imports: pulling in building blocks from other crates                   │
// └───────────────────────────────────────────────────────────────────────────┘

use async_trait::async_trait; // allows async fn in trait impls via macro magic
use futures::TryStreamExt; // adds try_collect() etc. to async streams
use humantime::Duration as HumanDuration; // parse "5s"/"10m" strings into Duration
use iggy_common::{DateTime, Utc}; // shared DateTime + UTC type alias in Iggy
use iggy_connector_sdk::{
    ConnectorState, Error, ProducedMessage, ProducedMessages, Schema, Source, source_connector,
};
use mongodb::{
    Client, Collection,
    bson::{Bson, Document, doc, oid::ObjectId},
    options::{ClientOptions, FindOptions},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

// This macro call effectively “registers” MongoDbSource as a source connector type
// with the Iggy connector SDK. It usually:
//   - implements some boilerplate
//   - wires up metadata about the connector type
source_connector!(MongoDbSource);

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Constants: default knobs + human-readable messages                      │
// └───────────────────────────────────────────────────────────────────────────┘

const DEFAULT_MAX_RETRIES: u32 = 3; // how many times to retry transient poll errors
const DEFAULT_RETRY_DELAY: &str = "1s"; // default retry backoff step
const DEFAULT_POLL_INTERVAL: &str = "10s"; // sleep between polls when idle
const DEFAULT_BATCH_SIZE: u32 = 1000; // how many docs to ask for in one poll
const CONNECTOR_NAME: &str = "MongoDB source"; // used in persisted state
const SUPPORTED_TRACKING_BSON_KINDS: &str = "int32, int64, double, string, object_id, date_time";

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ PayloadFormat: how we encode each ProducedMessage payload               │
// └───────────────────────────────────────────────────────────────────────────┘

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PayloadFormat {
    #[default]
    Json,   // default: serialize the document (or field) as JSON bytes
    Bson,   // raw BSON bytes
    String, // plain text representation
}

impl PayloadFormat {
    // Read from config string; be forgiving about casing and synonyms.
    fn from_config(s: Option<&str>) -> Self {
        match s.map(|s| s.to_lowercase()).as_deref() {
            Some("bson") | Some("binary") => PayloadFormat::Bson,
            Some("string") | Some("text") => PayloadFormat::String,
            _ => PayloadFormat::Json,
        }
    }

    // Map to the schema enum Iggy core understands.
    fn to_schema(self) -> Schema {
        match self {
            PayloadFormat::Json => Schema::Json,
            PayloadFormat::Bson => Schema::Raw,
            PayloadFormat::String => Schema::Text,
        }
    }
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Helper: detect “transient” errors worth retrying                        │
// └───────────────────────────────────────────────────────────────────────────┘

fn is_transient_error(error: &str) -> bool {
    let msg = error.to_lowercase();
    msg.contains("timeout")
        || msg.contains("network")
        || msg.contains("connection")
        || msg.contains("pool")
        || msg.contains("server selection")
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Helper: convert camelCase → snake_case for field names                  │
// └───────────────────────────────────────────────────────────────────────────┘

fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Expected vs actual document counts after delete/update                  │
// │ - this powers “did we touch as many docs as we thought we would?”      │
// └───────────────────────────────────────────────────────────────────────────┘

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedActualCountMismatch {
    None,
    Partial { expected: u64, actual: u64 },
    Complete { expected: u64 },
}

fn classify_expected_actual_mismatch(
    expected_count: u64,
    actual_count: u64,
) -> ExpectedActualCountMismatch {
    if expected_count == 0 || actual_count >= expected_count {
        ExpectedActualCountMismatch::None
    } else if actual_count == 0 {
        ExpectedActualCountMismatch::Complete {
            expected: expected_count,
        }
    } else {
        ExpectedActualCountMismatch::Partial {
            expected: expected_count,
            actual: actual_count,
        }
    }
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Offset conversion helpers                                               │
// │ - deal with different BSON types that can be used as tracking offsets   │
// └───────────────────────────────────────────────────────────────────────────┘

/// Converts an offset string to the appropriate BSON type for query comparison.
///
/// Why string? Because older / simpler configs might treat everything as a
/// string initially. We upgrade it into a more precise BSON value here.
///
/// Priority:
///   1. Parse as i64 if it looks numeric (this is explicit).
///   2. If tracking `_id` and it looks like a 24-char hex string, try ObjectId.
///   3. Otherwise, keep it as plain string.
fn convert_offset_value_to_bson(offset: &str, tracking_field: &str) -> Bson {
    // Try numeric first (highest priority - explicit numbers)
    if let Ok(n) = offset.parse::<i64>() {
        return Bson::Int64(n);
    }

    // Only try ObjectId conversion for _id field to avoid false positives
    // on custom string fields that happen to look like ObjectId hex
    if tracking_field == "_id"
        && offset.len() == 24
        && offset.chars().all(|c| c.is_ascii_hexdigit())
        && let Ok(oid) = ObjectId::parse_str(offset)
    {
        return Bson::ObjectId(oid);
    }

    // Fallback to string comparison
    Bson::String(offset.to_string())
}

// Helpful for human-readable logs about what BSON kind we’re dealing with.
fn describe_bson_tracking_kind_now(value: &Bson) -> &'static str {
    match value {
        Bson::Double(_) => "double",
        Bson::String(_) => "string",
        Bson::Array(_) => "array",
        Bson::Document(_) => "document",
        Bson::Boolean(_) => "boolean",
        Bson::Null => "null",
        Bson::RegularExpression(_) => "regular_expression",
        Bson::JavaScriptCode(_) => "javascript_code",
        Bson::JavaScriptCodeWithScope(_) => "javascript_code_with_scope",
        Bson::Int32(_) => "int32",
        Bson::Int64(_) => "int64",
        Bson::Timestamp(_) => "timestamp",
        Bson::Binary(_) => "binary",
        Bson::ObjectId(_) => "object_id",
        Bson::DateTime(_) => "date_time",
        Bson::Symbol(_) => "symbol",
        Bson::Decimal128(_) => "decimal128",
        Bson::Undefined => "undefined",
        Bson::MaxKey => "max_key",
        Bson::MinKey => "min_key",
        Bson::DbPointer(_) => "db_pointer",
    }
}

fn build_missing_field_error_now(collection_name: &str, tracking_field: &str) -> Error {
    Error::Storage(format!(
        "Tracking field '{tracking_field}' is missing in collection '{collection_name}'. Projection and query settings must keep the tracking field visible."
    ))
}

fn build_tracking_kind_error_now(
    collection_name: &str,
    tracking_field: &str,
    value: &Bson,
) -> Error {
    Error::Storage(format!(
        "Unsupported BSON kind '{}' for tracking field '{}' in collection '{}'. Supported kinds: {}.",
        describe_bson_tracking_kind_now(value),
        tracking_field,
        collection_name,
        SUPPORTED_TRACKING_BSON_KINDS
    ))
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ TrackingOffsetValue + PersistedTrackingOffset                           │
// │ - explicit, typed representation of offsets we store as state           │
// └───────────────────────────────────────────────────────────────────────────┘

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum TrackingOffsetValue {
    Int64(i64),
    Double(f64),
    String(String),
    ObjectIdHex(String),
    DateTimeMillis(i64),
}

impl TrackingOffsetValue {
    fn from_bson_value_now(value: &Bson) -> Result<Self, Error> {
        match value {
            Bson::Int32(v) => Ok(Self::Int64((*v).into())),
            Bson::Int64(v) => Ok(Self::Int64(*v)),
            Bson::Double(v) => Ok(Self::Double(*v)),
            Bson::String(value) => Ok(Self::String(value.clone())),
            Bson::ObjectId(object_id) => Ok(Self::ObjectIdHex(object_id.to_hex())),
            Bson::DateTime(value) => Ok(Self::DateTimeMillis(value.timestamp_millis())),
            _ => Err(Error::InvalidRecord),
        }
    }

    fn to_query_bson_now(&self) -> Bson {
        match self {
            Self::Int64(value) => Bson::Int64(*value),
            Self::Double(value) => Bson::Double(*value),
            Self::String(value) => Bson::String(value.clone()),
            Self::ObjectIdHex(value) => ObjectId::parse_str(value)
                .map(Bson::ObjectId)
                .unwrap_or_else(|_| Bson::String(value.clone())),
            Self::DateTimeMillis(value) => {
                Bson::DateTime(mongodb::bson::DateTime::from_millis(*value))
            }
        }
    }

    fn display_offset_value_now(&self) -> String {
        match self {
            Self::Int64(value) => value.to_string(),
            Self::Double(value) => value.to_string(),
            Self::String(value) => value.clone(),
            Self::ObjectIdHex(value) => value.clone(),
            Self::DateTimeMillis(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PersistedTrackingOffset {
    Typed(TrackingOffsetValue), // new, strongly-typed representation
    LegacyString(String),       // backward-compatible string form
}

impl PersistedTrackingOffset {
    fn from_bson_value_now(value: &Bson) -> Result<Self, Error> {
        Ok(Self::Typed(TrackingOffsetValue::from_bson_value_now(
            value,
        )?))
    }

    fn to_query_bson_now(&self, tracking_field: &str) -> Bson {
        match self {
            Self::Typed(value) => value.to_query_bson_now(),
            Self::LegacyString(value) => convert_offset_value_to_bson(value, tracking_field),
        }
    }

    fn display_offset_value_now(&self) -> String {
        match self {
            Self::Typed(value) => value.display_offset_value_now(),
            Self::LegacyString(value) => value.clone(),
        }
    }
}

impl Serialize for PersistedTrackingOffset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Typed(value) => value.serialize(serializer),
            Self::LegacyString(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PersistedTrackingOffset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PersistedTrackingOffsetWire {
            Typed(TrackingOffsetValue),
            LegacyString(String),
        }

        Ok(
            match PersistedTrackingOffsetWire::deserialize(deserializer)? {
                PersistedTrackingOffsetWire::Typed(value) => Self::Typed(value),
                PersistedTrackingOffsetWire::LegacyString(value) => Self::LegacyString(value),
            },
        )
    }
}

// Small test-only helper: simpler API when projection accidentally hides tracking field.
#[cfg(test)]
fn extract_tracking_offset_from_document(
    document: &Document,
    tracking_field: &str,
) -> Result<PersistedTrackingOffset, Error> {
    let bson_value = document.get(tracking_field).ok_or(Error::InvalidRecord)?;
    PersistedTrackingOffset::from_bson_value_now(bson_value)
}

fn extract_tracking_offset_with_context(
    document: &Document,
    collection_name: &str,
    tracking_field: &str,
) -> Result<PersistedTrackingOffset, Error> {
    let Some(bson_value) = document.get(tracking_field) else {
        return Err(build_missing_field_error_now(
            collection_name,
            tracking_field,
        ));
    };

    PersistedTrackingOffset::from_bson_value_now(bson_value)
        .map_err(|_| build_tracking_kind_error_now(collection_name, tracking_field, bson_value))
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Duplicate boundary logic                                                │
// │ - guard against non-unique tracking fields at batch edges              │
// └───────────────────────────────────────────────────────────────────────────┘

fn find_previous_distinct_offset(
    batch_offsets: &[PersistedTrackingOffset],
) -> Option<PersistedTrackingOffset> {
    let last_offset = batch_offsets.last()?;
    batch_offsets
        .iter()
        .rev()
        .skip(1)
        .find(|offset| *offset != last_offset)
        .cloned()
}

fn resolve_checkpoint_offset_for_batch(
    batch_offsets: &[PersistedTrackingOffset],
    extra_offset: Option<&PersistedTrackingOffset>,
    tracking_field: &str,
) -> Option<PersistedTrackingOffset> {
    let max_offset = batch_offsets.last().cloned();

    if tracking_field == "_id" {
        return max_offset;
    }

    let batch_max_offset = max_offset.as_ref()?;
    let Some(extra_offset) = extra_offset else {
        return max_offset;
    };

    if extra_offset != batch_max_offset {
        return max_offset;
    }

    find_previous_distinct_offset(batch_offsets)
}

#[derive(Debug, Clone, PartialEq)]
enum DuplicateBoundaryOutcome {
    None,
    Warn(PersistedTrackingOffset),
    Fail,
}

fn classify_duplicate_boundary_now(
    batch_offsets: &[PersistedTrackingOffset],
    extra_offset: Option<&PersistedTrackingOffset>,
    tracking_field: &str,
) -> DuplicateBoundaryOutcome {
    if tracking_field == "_id" {
        return DuplicateBoundaryOutcome::None;
    }

    let Some(boundary_offset) = batch_offsets.last() else {
        return DuplicateBoundaryOutcome::None;
    };
    let Some(extra_offset) = extra_offset else {
        return DuplicateBoundaryOutcome::None;
    };

    if extra_offset != boundary_offset {
        return DuplicateBoundaryOutcome::None;
    }

    if find_previous_distinct_offset(batch_offsets).is_none() {
        return DuplicateBoundaryOutcome::Fail;
    }

    DuplicateBoundaryOutcome::Warn(boundary_offset.clone())
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Core structs: MongoDbSource + config + in-memory state                  │
// └───────────────────────────────────────────────────────────────────────────┘

#[derive(Debug)]
pub struct MongoDbSource {
    pub id: u32,          // connector ID given by the host
    client: Option<Client>, // MongoDB client (None until open())
    config: MongoDbSourceConfig, // user-supplied config
    state: Mutex<SourceState>,  // internal state protected by async Mutex
    verbose: bool,              // whether to log more details
    retry_delay: Duration,      // base duration used for backoff retries
    poll_interval: Duration,    // how long we sleep between polls
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDbSourceConfig {
    pub connection_uri: String,
    pub database: String,
    pub collection: String,
    pub poll_interval: Option<String>,
    pub batch_size: Option<u32>,
    pub max_pool_size: Option<u32>,
    pub tracking_field: Option<String>,
    pub initial_offset: Option<String>,
    pub query_filter: Option<String>,
    pub projection: Option<String>,
    pub snake_case_fields: Option<bool>,
    pub include_metadata: Option<bool>,
    pub delete_after_read: Option<bool>,
    pub processed_field: Option<String>,
    pub payload_field: Option<String>,
    pub payload_format: Option<String>,
    pub verbose_logging: Option<bool>,
    pub max_retries: Option<u32>,
    pub retry_delay: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    last_poll_time: DateTime<Utc>,                 // when we last polled Mongo
    tracking_offsets: HashMap<String, PersistedTrackingOffset>, // per-collection offsets
    processed_documents: u64,                      // running count for diagnostics
}

#[derive(Debug, Clone)]
struct PendingBatchState {
    collection: String,                      // which collection this batch came from
    checkpoint_offset: Option<PersistedTrackingOffset>, // offset to commit if acked
    processed_count: u64,                    // how many docs in this batch
}

#[derive(Debug)]
struct SourceState {
    committed_state: State,                // fully committed progress
    pending_batch: Option<PendingBatchState>, // currently in-flight batch (not yet committed)
}

#[derive(Debug)]
struct PreparedPollBatch {
    messages: Vec<ProducedMessage>,         // what we’ll send to Iggy
    pending_batch: Option<PendingBatchState>, // metadata for commit
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Constructor + small helpers                                             │
// └───────────────────────────────────────────────────────────────────────────┘

impl MongoDbSource {
    pub fn new(id: u32, config: MongoDbSourceConfig, state: Option<ConnectorState>) -> Self {
        let verbose = config.verbose_logging.unwrap_or(false);

        // Parse retry delay from config like "1s", else default to 1 second.
        let delay_str = config.retry_delay.as_deref().unwrap_or(DEFAULT_RETRY_DELAY);
        let retry_delay = HumanDuration::from_str(delay_str)
            .map(|duration| duration.into())
            .unwrap_or_else(|_| Duration::from_secs(1));

        // Parse poll interval from config, else default to 10 seconds.
        let poll_str = config
            .poll_interval
            .as_deref()
            .unwrap_or(DEFAULT_POLL_INTERVAL);
        let poll_interval = HumanDuration::from_str(poll_str)
            .map(|duration| duration.into())
            .unwrap_or_else(|_| Duration::from_secs(10));

        // Restore persisted state or seed from initial_offset when none exists.
        //
        // The idea:
        //   - If Iggy saved a serialized State before, deserialize it.
        //   - Otherwise, start fresh, optionally seeding an initial offset for the collection.
        let initial_state = state
            .and_then(|s| s.deserialize(CONNECTOR_NAME, id))
            .unwrap_or_else(|| {
                let mut offsets = HashMap::new();
                if let Some(offset) = &config.initial_offset {
                    offsets.insert(
                        config.collection.clone(),
                        PersistedTrackingOffset::LegacyString(offset.clone()),
                    );
                }
                State {
                    last_poll_time: Utc::now(),
                    tracking_offsets: offsets,
                    processed_documents: 0,
                }
            });

        // Safety net: if we restored any legacy string offsets, warn the user so they
        // know that some reinterpretation of types might happen until a new checkpoint
        // is written using the newer typed representation.
        if initial_state
            .tracking_offsets
            .values()
            .any(|offset| matches!(offset, PersistedTrackingOffset::LegacyString(_)))
        {
            warn!(
                collection = %config.collection,
                "Loaded legacy untyped tracking offset state; numeric-looking strings and _id-like strings may be reinterpreted until a typed checkpoint is saved"
            );
        }

        MongoDbSource {
            id,
            client: None,
            config,
            state: Mutex::new(SourceState {
                committed_state: initial_state,
                pending_batch: None,
            }),
            verbose,
            retry_delay,
            poll_interval,
        }
    }

    // Helper: get typed collection handle or fail with clear error if client not ready.
    fn get_collection(&self) -> Result<Collection<Document>, Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| Error::InitError("MongoDB client not initialized".to_string()))?;

        Ok(client
            .database(&self.config.database)
            .collection(&self.config.collection))
    }

    // Turn internal `State` into ConnectorState so Iggy can persist it.
    fn serialize_state(&self, state: &State) -> Option<ConnectorState> {
        ConnectorState::serialize(state, CONNECTOR_NAME, self.id)
    }

    // Get max retries, defaulting to the constant.
    fn get_max_retries(&self) -> u32 {
        self.config.max_retries.unwrap_or(DEFAULT_MAX_RETRIES)
    }
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Source trait implementation: Iggy’s expected interface                   │
// └───────────────────────────────────────────────────────────────────────────┘

#[async_trait]
impl Source for MongoDbSource {
    // OPEN:
    // - Build MongoDB client from URI
    // - Apply pool options
    // - Ping server
    // - Warn if collection doesn’t exist (but don’t fail)
    async fn open(&mut self) -> Result<(), Error> {
        info!(
            "Opening MongoDB source connector with ID: {}. Database: {}. Collection: {}",
            self.id, self.config.database, self.config.collection
        );

        // Parse connection string and build client options
        let mut client_options = ClientOptions::parse(&self.config.connection_uri)
            .await
            .map_err(|e| Error::InitError(format!("Failed to parse connection URI: {e}")))?;

        // Configure connection pool
        if let Some(max_pool_size) = self.config.max_pool_size {
            client_options.max_pool_size = Some(max_pool_size);
        }

        // Build client
        let client = Client::with_options(client_options)
            .map_err(|e| Error::InitError(format!("Failed to create MongoDB client: {e}")))?;

        // Ping server to verify connectivity
        client
            .database("admin")
            .run_command(doc! {"ping": 1})
            .await
            .map_err(|e| Error::InitError(format!("MongoDB ping failed: {e}")))?;

        self.client = Some(client);

        // Validate collection exists (warn if missing, do not fail)
        self.validate_collection().await?;

        info!(
            "MongoDB source connector with ID: {} opened successfully",
            self.id
        );
        Ok(())
    }

    // POLL:
    // - sleep for configured interval
    // - call `poll_collection()` which retries on transient errors
    // - stash state for later commit/discard
    // - return ProducedMessages with appropriate schema
    async fn poll(&self) -> Result<ProducedMessages, Error> {
        let poll_interval = self.poll_interval;
        tokio::time::sleep(poll_interval).await;

        let prepared_batch = self.poll_collection().await?;

        let mut state = self.state.lock().await;
        if state.pending_batch.is_some() {
            // Invariant: we don’t want to poll again if there is already an uncommitted batch.
            return Err(Error::InvalidState);
        }

        state.committed_state.last_poll_time = Utc::now();
        state.pending_batch = prepared_batch.pending_batch;
        let processed_documents = state.committed_state.processed_documents;

        if self.verbose {
            info!(
                "MongoDB source connector ID: {} produced {} messages. Total processed: {}",
                self.id,
                prepared_batch.messages.len(),
                processed_documents
            );
        } else {
            debug!(
                "MongoDB source connector ID: {} produced {} messages. Total processed: {}",
                self.id,
                prepared_batch.messages.len(),
                processed_documents
            );
        }

        // Derive schema from payload_format config
        let payload_format = PayloadFormat::from_config(self.config.payload_format.as_deref());
        let schema = payload_format.to_schema();

        Ok(ProducedMessages {
            schema,
            messages: prepared_batch.messages,
            state: None,
        })
    }

    // COMMIT:
    // - apply side effects to Mongo (delete or mark processed if configured)
    // - advance committed offsets + document count
    // - serialize state for persistence
    async fn commit_polled_messages_now(&self) -> Result<Option<ConnectorState>, Error> {
        let pending_batch = {
            let state = self.state.lock().await;
            state.pending_batch.clone()
        };

        if let Some(pending_batch) = pending_batch.as_ref() {
            if self.config.delete_after_read.unwrap_or(false) {
                self.delete_processed_documents(
                    pending_batch.checkpoint_offset.as_ref(),
                    pending_batch.processed_count,
                )
                .await?;
            } else if let Some(processed_field) = &self.config.processed_field {
                self.mark_documents_processed(
                    processed_field,
                    pending_batch.checkpoint_offset.as_ref(),
                    pending_batch.processed_count,
                )
                .await?;
            }
        }

        let mut state = self.state.lock().await;
        if let Some(pending_batch) = state.pending_batch.take() {
            if let Some(offset) = pending_batch.checkpoint_offset {
                state
                    .committed_state
                    .tracking_offsets
                    .insert(pending_batch.collection, offset);
            }
            state.committed_state.processed_documents += pending_batch.processed_count;
        }

        Ok(self.serialize_state(&state.committed_state))
    }

    // DISCARD:
    // - clear pending batch without touching committed offsets
    async fn discard_polled_messages_now(&self) -> Result<(), Error> {
        let mut state = self.state.lock().await;
        state.pending_batch = None;
        Ok(())
    }

    // CLOSE:
    // - drop client
    // - log summary
    async fn close(&mut self) -> Result<(), Error> {
        info!("Closing MongoDB source connector with ID: {}", self.id);

        // Client will be dropped automatically
        self.client.take();

        let state = self.state.lock().await;
        info!(
            "MongoDB source connector ID: {} closed. Total documents processed: {}",
            self.id, state.committed_state.processed_documents
        );
        Ok(())
    }
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Internal methods: validation, polling, document conversion              │
// └───────────────────────────────────────────────────────────────────────────┘

impl MongoDbSource {
    // Just checks if the collection name exists; warns if it doesn’t.
    async fn validate_collection(&self) -> Result<(), Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| Error::InitError("MongoDB client not initialized".to_string()))?;

        let db = client.database(&self.config.database);

        // List collection names
        let collection_names = db
            .list_collection_names()
            .await
            .map_err(|e| Error::InitError(format!("Failed to list collections: {e}")))?;

        if !collection_names.contains(&self.config.collection) {
            warn!(
                "Collection '{}.{}' does not exist yet - polling will return empty results until the collection is created",
                self.config.database, self.config.collection
            );
        }

        Ok(())
    }

    /// Retry wrapper: calls execute_poll() with transient error retry logic.
    ///
    /// ASCII timeline for retries (simplified):
    ///
    ///   attempt 1 ──fail───► sleep(retry_delay * 1)
    ///   attempt 2 ──fail───► sleep(retry_delay * 2)
    ///   attempt 3 ──fail───► give up if max_retries == 3
    ///
    async fn poll_collection(&self) -> Result<PreparedPollBatch, Error> {
        let max_retries = self.get_max_retries();
        let mut attempts = 0u32;
        loop {
            match self.execute_poll().await {
                Ok(msgs) => return Ok(msgs),
                Err(e) if is_transient_error(&e.to_string()) && attempts < max_retries => {
                    attempts += 1;
                    warn!("Poll failed (attempt {attempts}/{max_retries}): {e}. Retrying...");
                    tokio::time::sleep(self.retry_delay * attempts).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Core poll implementation: build filter, run find(), convert documents.
    ///
    /// Roughly:
    ///
    ///   1. figure out last offset (bookmark)
    ///   2. build Mongo filter > last_offset
    ///   3. execute find() with sort + projection + batch size
    ///   4. convert each document to `ProducedMessage`
    ///   5. compute which offset to save as checkpoint
    ///
    async fn execute_poll(&self) -> Result<PreparedPollBatch, Error> {
        let collection = self.get_collection()?;

        // Build query filter
        let tracking_field = self.config.tracking_field.as_deref().unwrap_or("_id");

        let state = self.state.lock().await;
        let last_offset = state
            .committed_state
            .tracking_offsets
            .get(&self.config.collection)
            .cloned();
        drop(state);

        let filter = self.build_filter_document(
            last_offset.as_ref(),
            tracking_field,
            "$gt",
            self.config.processed_field.as_deref(),
        )?;

        // Build projection if configured
        let projection = if let Some(projection_str) = &self.config.projection {
            Some(
                serde_json::from_str::<Document>(projection_str)
                    .map_err(|_e| Error::InvalidConfig)?,
            )
        } else {
            None
        };

        // Build find options
        let configured_batch_size = self.config.batch_size.unwrap_or(DEFAULT_BATCH_SIZE);
        let mut find_options = FindOptions::default();
        // We ask for N+1 docs; if we get that extra one it helps us detect boundary issues
        // when tracking_field is not unique.
        find_options.limit = Some(configured_batch_size.saturating_add(1) as i64);
        find_options.sort = Some(doc! {tracking_field: 1});
        if let Some(proj) = projection {
            find_options.projection = Some(proj);
        }

        // Execute query
        let cursor = collection
            .find(filter)
            .with_options(find_options)
            .await
            .map_err(|e| Error::Storage(format!("Failed to query collection: {e}")))?;

        let mut documents = cursor
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch documents: {e}")))?;

        let configured_batch_size = configured_batch_size as usize;
        let extra_document = if documents.len() > configured_batch_size {
            documents.pop()
        } else {
            None
        };

        // Convert documents to messages
        let mut messages = Vec::with_capacity(documents.len());
        let mut batch_offsets = Vec::with_capacity(documents.len());

        for doc in documents {
            let offset = extract_tracking_offset_with_context(
                &doc,
                &self.config.collection,
                tracking_field,
            )?;
            batch_offsets.push(offset);

            let message = self.document_to_message(doc, tracking_field).await?;
            messages.push(message);
        }

        let extra_offset = extra_document
            .as_ref()
            .map(|doc| {
                extract_tracking_offset_with_context(doc, &self.config.collection, tracking_field)
            })
            .transpose()?;
        let duplicate_boundary =
            classify_duplicate_boundary_now(&batch_offsets, extra_offset.as_ref(), tracking_field);
        let checkpoint_offset = resolve_checkpoint_offset_for_batch(
            &batch_offsets,
            extra_offset.as_ref(),
            tracking_field,
        );

        match duplicate_boundary {
            DuplicateBoundaryOutcome::None => {}
            DuplicateBoundaryOutcome::Warn(boundary_offset) => {
                warn!(
                    collection = %self.config.collection,
                    tracking_field = %tracking_field,
                    boundary_offset = %boundary_offset.display_offset_value_now(),
                    "Detected duplicate tracking value at batch boundary; rolling checkpoint back to avoid skipping equal offsets"
                );
            }
            DuplicateBoundaryOutcome::Fail => {
                return Err(Error::Storage(format!(
                    "Tracking field '{}' has duplicate values at the batch boundary in collection '{}'. Reduce batch_size or choose a unique tracking field.",
                    tracking_field, self.config.collection
                )));
            }
        }

        let pending_batch = (!messages.is_empty()).then(|| PendingBatchState {
            collection: self.config.collection.clone(),
            checkpoint_offset,
            processed_count: messages.len() as u64,
        });

        Ok(PreparedPollBatch {
            messages,
            pending_batch,
        })
    }

    // Turn a MongoDB document into a ProducedMessage.
    //
    // Steps:
    //   1. Derive timestamp (prefer embedded ObjectId timestamp if tracking by _id).
    //   2. Optionally inject metadata fields.
    //   3. Optionally rename fields to snake_case.
    //   4. Extract payload (whole doc or specific field).
    //   5. Encode payload as JSON/BSON/String based on PayloadFormat.
    async fn document_to_message(
        &self,
        mut doc: Document,
        tracking_field: &str,
    ) -> Result<ProducedMessage, Error> {
        // Extract timestamp before any mutation of the document.
        // For _id ObjectId, use the embedded creation timestamp.
        // ObjectId::timestamp() returns bson::DateTime whose timestamp_millis() gives ms since epoch.
        let timestamp_ms: u64 = if tracking_field == "_id" {
            match doc.get("_id") {
                Some(Bson::ObjectId(oid)) => {
                    let bson_dt = oid.timestamp();
                    bson_dt.timestamp_millis() as u64
                }
                _ => Utc::now().timestamp_millis() as u64,
            }
        } else {
            Utc::now().timestamp_millis() as u64
        };

        // Inject metadata fields when include_metadata is enabled
        if self.config.include_metadata.unwrap_or(false) {
            doc.insert("_iggy_source_collection", self.config.collection.as_str());
            doc.insert("_iggy_poll_timestamp", Utc::now().to_rfc3339());
        }

        // Apply snake_case conversion to field names when enabled
        let doc = if self.config.snake_case_fields.unwrap_or(false) {
            let mut converted = Document::new();
            for (key, value) in doc {
                converted.insert(to_snake_case(&key), value);
            }
            converted
        } else {
            doc
        };

        // Determine payload format
        let payload_format = PayloadFormat::from_config(self.config.payload_format.as_deref());

        // If payload_field is specified, extract that field; otherwise use entire doc
        let payload_bytes = if let Some(payload_field) = &self.config.payload_field {
            let resolved_payload_field = if self.config.snake_case_fields.unwrap_or(false) {
                to_snake_case(payload_field)
            } else {
                payload_field.clone()
            };
            let payload_value = doc
                .get(&resolved_payload_field)
                .ok_or(Error::InvalidRecord)?;

            match payload_format {
                PayloadFormat::Json => {
                    serde_json::to_vec(payload_value).map_err(|_| Error::InvalidRecord)?
                }
                PayloadFormat::Bson => {
                    let mut buf = Vec::new();
                    let bson_doc = doc! { resolved_payload_field: payload_value.clone() };
                    bson_doc
                        .to_writer(&mut buf)
                        .map_err(|_| Error::InvalidRecord)?;
                    buf
                }
                PayloadFormat::String => {
                    let s = format!("{payload_value}");
                    s.into_bytes()
                }
            }
        } else {
            match payload_format {
                PayloadFormat::Json => {
                    serde_json::to_vec(&doc).map_err(|_| Error::InvalidRecord)?
                }
                PayloadFormat::Bson => {
                    let mut buf = Vec::new();
                    doc.to_writer(&mut buf).map_err(|_| Error::InvalidRecord)?;
                    buf
                }
                PayloadFormat::String => {
                    let s = serde_json::to_string(&doc).map_err(|_| Error::InvalidRecord)?;
                    s.into_bytes()
                }
            }
        };

        Ok(ProducedMessage {
            id: Some(Uuid::new_v4().as_u128()),
            headers: None,
            checksum: None,
            timestamp: Some(timestamp_ms),
            origin_timestamp: Some(timestamp_ms),
            payload: payload_bytes,
        })
    }

    // Build the MongoDB filter document for polling / marking / deleting.
    //
    // It combines:
    //   - tracking clause       (e.g. { seq: { "$gt": 42 } })
    //   - query_filter (config) (e.g. { tenant: "alpha" })
    //   - processed_field bool  (e.g. { processed: false })
    //
    // If more than one clause exists, they’re AND-ed together.
    fn build_filter_document(
        &self,
        last_offset: Option<&PersistedTrackingOffset>,
        tracking_field: &str,
        tracking_operator: &str,
        processed_field: Option<&str>,
    ) -> Result<Document, Error> {
        let mut clauses = Vec::new();

        if let Some(offset) = last_offset {
            let offset_bson = offset.to_query_bson_now(tracking_field);
            clauses.push(doc! {tracking_field: {tracking_operator: offset_bson}});
        }

        if let Some(query_filter_str) = &self.config.query_filter {
            clauses.push(
                serde_json::from_str::<Document>(query_filter_str)
                    .map_err(|_e| Error::InvalidConfig)?,
            );
        }

        if let Some(processed_field) = processed_field {
            clauses.push(doc! {processed_field: false});
        }

        Ok(match clauses.len() {
            0 => doc! {},
            1 => clauses.pop().unwrap_or_default(),
            _ => doc! {
                "$and": clauses.into_iter().map(Bson::Document).collect::<Vec<_>>()
            },
        })
    }

    // If `delete_after_read` is enabled, this physically removes documents
    // up to and including the current offset.
    async fn delete_processed_documents(
        &self,
        current_offset: Option<&PersistedTrackingOffset>,
        expected_count: u64,
    ) -> Result<(), Error> {
        let collection = self.get_collection()?;
        let tracking_field = self.config.tracking_field.as_deref().unwrap_or("_id");

        if let Some(offset) = current_offset {
            let delete_filter =
                self.build_filter_document(Some(offset), tracking_field, "$lte", None)?;

            let result = collection.delete_many(delete_filter).await.map_err(|e| {
                Error::Storage(format!("Failed to delete processed documents: {e}"))
            })?;

            match classify_expected_actual_mismatch(expected_count, result.deleted_count) {
                ExpectedActualCountMismatch::None => {
                    debug!(
                        "Deleted {} processed documents up to offset: {}",
                        result.deleted_count,
                        offset.display_offset_value_now()
                    );
                }
                ExpectedActualCountMismatch::Partial { expected, actual } => {
                    tracing::warn!(
                        collection = %self.config.collection,
                        expected,
                        actual,
                        offset = %offset.display_offset_value_now(),
                        "delete_processed_documents: partial mismatch (deleted fewer documents than expected)"
                    );
                }
                ExpectedActualCountMismatch::Complete { expected } => {
                    tracing::warn!(
                        collection = %self.config.collection,
                        expected,
                        actual = result.deleted_count,
                        offset = %offset.display_offset_value_now(),
                        "delete_processed_documents: complete mismatch (expected deletions but got 0)"
                    );
                }
            }
        }

        Ok(())
    }

    // If `processed_field` is configured, we mark documents as processed instead
    // of deleting them (soft acknowledgement).
    async fn mark_documents_processed(
        &self,
        processed_field: &str,
        current_offset: Option<&PersistedTrackingOffset>,
        expected_count: u64,
    ) -> Result<(), Error> {
        let collection = self.get_collection()?;
        let tracking_field = self.config.tracking_field.as_deref().unwrap_or("_id");

        if let Some(offset) = current_offset {
            let update_filter = self.build_filter_document(
                Some(offset),
                tracking_field,
                "$lte",
                Some(processed_field),
            )?;
            let update = doc! {"$set": {processed_field: true}};

            let result = collection
                .update_many(update_filter, update)
                .await
                .map_err(|e| {
                    Error::Storage(format!("Failed to mark documents as processed: {e}"))
                })?;

            match classify_expected_actual_mismatch(expected_count, result.matched_count) {
                ExpectedActualCountMismatch::None => {
                    debug!(
                        "Marked {} documents as processed up to offset: {}",
                        result.matched_count,
                        offset.display_offset_value_now()
                    );
                }
                ExpectedActualCountMismatch::Partial { expected, actual } => {
                    tracing::warn!(
                        collection = %self.config.collection,
                        processed_field = %processed_field,
                        expected,
                        actual,
                        offset = %offset.display_offset_value_now(),
                        "mark_documents_processed: partial mismatch (matched fewer documents than expected)"
                    );
                }
                ExpectedActualCountMismatch::Complete { expected } => {
                    tracing::warn!(
                        collection = %self.config.collection,
                        processed_field = %processed_field,
                        expected,
                        actual = result.matched_count,
                        offset = %offset.display_offset_value_now(),
                        "mark_documents_processed: complete mismatch (expected matches but got 0)"
                    );
                }
            }
        }

        Ok(())
    }
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │ Tests: keep same tests so behaviour matches original                     │
// │ These are left mostly unchanged; comments focus on intent.               │
// └───────────────────────────────────────────────────────────────────────────┘

#[cfg(test)]
mod tests {
    use super::*;

    fn given_int_tracking_offset(value: i64) -> PersistedTrackingOffset {
        PersistedTrackingOffset::Typed(TrackingOffsetValue::Int64(value))
    }

    fn given_default_config() -> MongoDbSourceConfig {
        MongoDbSourceConfig {
            connection_uri: "mongodb://localhost:27017".to_string(),
            database: "test_db".to_string(),
            collection: "test_collection".to_string(),
            poll_interval: None,
            batch_size: None,
            max_pool_size: None,
            tracking_field: None,
            initial_offset: None,
            query_filter: None,
            projection: None,
            snake_case_fields: None,
            include_metadata: None,
            delete_after_read: None,
            processed_field: None,
            payload_field: None,
            payload_format: None,
            verbose_logging: None,
            max_retries: None,
            retry_delay: None,
        }
    }

    // ---- Constructor and config tests ----

    #[test]
    fn given_valid_config_should_create_instance() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);

        assert_eq!(source.id, 1);
        assert!(source.client.is_none());
        assert!(!source.verbose);
    }

    #[test]
    fn given_default_config_should_use_default_max_retries() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);
        assert_eq!(source.get_max_retries(), DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn given_custom_max_retries_should_use_configured_value() {
        let mut config = given_default_config();
        config.max_retries = Some(5);
        let source = MongoDbSource::new(1, config, None);
        assert_eq!(source.get_max_retries(), 5);
    }

    #[test]
    fn given_verbose_enabled_should_set_verbose_flag() {
        let mut config = given_default_config();
        config.verbose_logging = Some(true);
        let source = MongoDbSource::new(1, config, None);
        assert!(source.verbose);
    }

    #[test]
    fn given_default_poll_interval_should_be_ten_seconds() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);
        assert_eq!(source.poll_interval, Duration::from_secs(10));
    }

    #[test]
    fn given_custom_poll_interval_should_parse_humantime() {
        let mut config = given_default_config();
        config.poll_interval = Some("5s".to_string());
        let source = MongoDbSource::new(1, config, None);
        assert_eq!(source.poll_interval, Duration::from_secs(5));
    }

    #[test]
    fn given_no_batch_size_should_use_default_via_constant() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);
        assert_eq!(source.config.batch_size, None);
    }

    #[test]
    fn given_custom_batch_size_should_store_in_config() {
        let mut config = given_default_config();
        config.batch_size = Some(500);
        let source = MongoDbSource::new(1, config, None);
        assert_eq!(source.config.batch_size, Some(500));
    }

    #[test]
    fn given_default_tracking_field_should_be_none_in_config() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);
        assert_eq!(source.config.tracking_field, None);
    }

    #[test]
    fn given_custom_tracking_field_should_store_in_config() {
        let mut config = given_default_config();
        config.tracking_field = Some("custom_id".to_string());
        let source = MongoDbSource::new(1, config, None);
        assert_eq!(source.config.tracking_field, Some("custom_id".to_string()));
    }

    #[test]
    fn given_no_persisted_state_should_start_with_empty_offsets() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);

        let state = source.state.try_lock().unwrap();
        assert_eq!(state.committed_state.processed_documents, 0);
        assert!(state.committed_state.tracking_offsets.is_empty());
        assert!(state.pending_batch.is_none());
    }

    #[test]
    fn given_initial_offset_with_no_persisted_state_should_seed_tracking() {
        let mut config = given_default_config();
        config.initial_offset = Some("63f5b2a0c1234567890abcde".to_string());
        let source = MongoDbSource::new(1, config.clone(), None);

        let state = source.state.try_lock().unwrap();
        assert_eq!(
            state
                .committed_state
                .tracking_offsets
                .get(&config.collection),
            Some(&PersistedTrackingOffset::LegacyString(
                "63f5b2a0c1234567890abcde".to_string()
            ))
        );
    }

    #[test]
    fn given_no_initial_offset_should_start_from_beginning() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config.clone(), None);

        let state = source.state.try_lock().unwrap();
        assert!(
            !state
                .committed_state
                .tracking_offsets
                .contains_key(&config.collection)
        );
    }

    #[test]
    fn given_valid_state_should_serialize_to_connector_state() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);

        let state = source.state.try_lock().unwrap();
        let connector_state = source.serialize_state(&state.committed_state);

        assert!(connector_state.is_some());
    }

    // ---- PayloadFormat tests ----

    #[test]
    fn given_json_format_string_should_return_json_variant() {
        assert_eq!(
            PayloadFormat::from_config(Some("json")),
            PayloadFormat::Json
        );
        assert_eq!(
            PayloadFormat::from_config(Some("JSON")),
            PayloadFormat::Json
        );
    }

    #[test]
    fn given_bson_format_string_should_return_bson_variant() {
        assert_eq!(
            PayloadFormat::from_config(Some("bson")),
            PayloadFormat::Bson
        );
        assert_eq!(
            PayloadFormat::from_config(Some("binary")),
            PayloadFormat::Bson
        );
        assert_eq!(
            PayloadFormat::from_config(Some("BSON")),
            PayloadFormat::Bson
        );
    }

    #[test]
    fn given_string_format_string_should_return_string_variant() {
        assert_eq!(
            PayloadFormat::from_config(Some("string")),
            PayloadFormat::String
        );
        assert_eq!(
            PayloadFormat::from_config(Some("text")),
            PayloadFormat::String
        );
        assert_eq!(
            PayloadFormat::from_config(Some("TEXT")),
            PayloadFormat::String
        );
    }

    #[test]
    fn given_unknown_format_should_default_to_json() {
        assert_eq!(
            PayloadFormat::from_config(Some("unknown")),
            PayloadFormat::Json
        );
        assert_eq!(PayloadFormat::from_config(None), PayloadFormat::Json);
    }

    #[test]
    fn given_json_format_should_return_schema_json() {
        assert_eq!(PayloadFormat::Json.to_schema(), Schema::Json);
    }

    #[test]
    fn given_bson_format_should_return_schema_raw() {
        assert_eq!(PayloadFormat::Bson.to_schema(), Schema::Raw);
    }

    #[test]
    fn given_string_format_should_return_schema_text() {
        assert_eq!(PayloadFormat::String.to_schema(), Schema::Text);
    }

    // ---- snake_case tests ----

    #[test]
    fn given_camel_case_input_should_convert_to_snake_case() {
        assert_eq!(to_snake_case("firstName"), "first_name");
        assert_eq!(to_snake_case("lastName"), "last_name");
        assert_eq!(to_snake_case("createdAt"), "created_at");
    }

    #[test]
    fn given_already_snake_case_should_remain_unchanged() {
        assert_eq!(to_snake_case("first_name"), "first_name");
        assert_eq!(to_snake_case("_id"), "_id");
    }

    #[test]
    fn given_single_word_lowercase_should_remain_unchanged() {
        assert_eq!(to_snake_case("name"), "name");
    }

    #[test]
    fn given_leading_uppercase_should_lowercase_without_leading_underscore() {
        assert_eq!(to_snake_case("Name"), "name");
    }

    // ---- is_transient_error tests ----

    #[test]
    fn given_timeout_error_message_should_be_transient() {
        assert!(is_transient_error("connection timeout occurred"));
        assert!(is_transient_error("operation timed out: timeout"));
    }

    #[test]
    fn given_network_error_message_should_be_transient() {
        assert!(is_transient_error("network failure detected"));
    }

    #[test]
    fn given_connection_error_message_should_be_transient() {
        assert!(is_transient_error("connection refused"));
    }

    // ---- classify_expected_actual_mismatch tests ----

    #[test]
    fn given_zero_expected_should_have_no_mismatch() {
        let result = classify_expected_actual_mismatch(0, 0);
        assert_eq!(result, ExpectedActualCountMismatch::None);
    }

    #[test]
    fn given_actual_at_least_expected_should_have_no_mismatch() {
        assert_eq!(
            classify_expected_actual_mismatch(5, 5),
            ExpectedActualCountMismatch::None
        );
        assert_eq!(
            classify_expected_actual_mismatch(5, 6),
            ExpectedActualCountMismatch::None
        );
    }

    #[test]
    fn given_zero_actual_with_expected_should_have_complete_mismatch() {
        let result = classify_expected_actual_mismatch(3, 0);
        assert_eq!(
            result,
            ExpectedActualCountMismatch::Complete { expected: 3 }
        );
    }

    #[test]
    fn given_partial_actual_with_expected_should_have_partial_mismatch() {
        let result = classify_expected_actual_mismatch(7, 4);
        assert_eq!(
            result,
            ExpectedActualCountMismatch::Partial {
                expected: 7,
                actual: 4
            }
        );
    }

    #[test]
    fn given_pool_error_message_should_be_transient() {
        assert!(is_transient_error("connection pool exhausted"));
    }

    #[test]
    fn given_server_selection_error_should_be_transient() {
        assert!(is_transient_error("server selection timeout"));
    }

    #[test]
    fn given_auth_failure_should_not_be_transient() {
        assert!(!is_transient_error(
            "authentication failed: bad credentials"
        ));
    }

    #[test]
    fn given_duplicate_key_error_should_not_be_transient() {
        assert!(!is_transient_error("duplicate key error on collection"));
    }

    #[test]
    fn given_invalid_bson_error_should_not_be_transient() {
        assert!(!is_transient_error("invalid bson: unexpected end of data"));
    }

    // ---- convert_offset_value_to_bson tests ----

    #[test]
    fn given_numeric_offset_should_return_int64_bson() {
        let result = convert_offset_value_to_bson("42", "_id");
        assert!(matches!(result, Bson::Int64(42)));
    }

    #[test]
    fn given_objectid_hex_should_return_objectid_bson() {
        let result = convert_offset_value_to_bson("507f1f77bcf86cd799439011", "_id");
        match result {
            Bson::ObjectId(oid) => {
                assert_eq!(oid.to_hex(), "507f1f77bcf86cd799439011");
            }
            _ => panic!("Expected ObjectId, got {:?}", result),
        }
    }

    #[test]
    fn given_lowercase_objectid_hex_should_return_objectid_bson() {
        let result = convert_offset_value_to_bson("507f1f77bcf86cd799439011", "_id");
        assert!(matches!(result, Bson::ObjectId(_)));
    }

    #[test]
    fn given_uppercase_objectid_hex_should_return_objectid_bson() {
        let result = convert_offset_value_to_bson("507F1F77BCF86CD799439011", "_id");
        assert!(matches!(result, Bson::ObjectId(_)));
    }

    #[test]
    fn given_invalid_objectid_hex_wrong_length_should_return_string() {
        // 23 chars instead of 24
        let result = convert_offset_value_to_bson("507f1f77bcf86cd79943901", "_id");
        assert!(matches!(result, Bson::String(_)));
    }

    #[test]
    fn given_non_hex_string_should_return_string_bson() {
        let result = convert_offset_value_to_bson("not-a-hex-string-!!!!", "_id");
        match result {
            Bson::String(s) => assert_eq!(s, "not-a-hex-string-!!!!"),
            _ => panic!("Expected String, got {:?}", result),
        }
    }

    #[test]
    fn given_timestamp_string_should_return_string_bson() {
        let result = convert_offset_value_to_bson("2024-01-15T10:30:00Z", "_id");
        assert!(matches!(result, Bson::String(_)));
    }

    #[test]
    fn given_objectid_hex_with_non_id_field_should_return_string() {
        // When tracking_field is NOT "_id", should NOT convert to ObjectId
        let result = convert_offset_value_to_bson("507f1f77bcf86cd799439011", "custom_id");
        assert!(
            matches!(result, Bson::String(_)),
            "Expected String when tracking_field is not _id, got {:?}",
            result
        );
    }

    #[test]
    fn query_filter_scopes_mark_delete_side_effects() {
        let mut config = given_default_config();
        config.query_filter = Some(r#"{"tenant":"alpha","kind":"event"}"#.to_string());
        let source = MongoDbSource::new(1, config, None);

        let filter = source
            .build_filter_document(Some(&given_int_tracking_offset(42)), "seq", "$lte", None)
            .expect("filter should build");

        let clauses = filter
            .get_array("$and")
            .expect("filter should compose clauses with $and");
        assert_eq!(clauses.len(), 2);

        let tracking_clause = clauses[0]
            .as_document()
            .expect("tracking clause should be a document");
        let seq = tracking_clause
            .get_document("seq")
            .expect("seq filter should be present");
        assert_eq!(seq.get("$lte"), Some(&Bson::Int64(42)));

        let query_clause = clauses[1]
            .as_document()
            .expect("query clause should be a document");
        assert_eq!(
            query_clause.get("tenant"),
            Some(&Bson::String("alpha".to_string()))
        );
        assert_eq!(
            query_clause.get("kind"),
            Some(&Bson::String("event".to_string()))
        );
    }

    #[test]
    fn query_filter_does_not_overwrite_tracking_clause() {
        let mut config = given_default_config();
        config.query_filter = Some(r#"{"seq":{"$lt":100},"tenant":"alpha"}"#.to_string());
        let source = MongoDbSource::new(1, config, None);

        let filter = source
            .build_filter_document(Some(&given_int_tracking_offset(42)), "seq", "$gt", None)
            .expect("filter should build");

        let clauses = filter
            .get_array("$and")
            .expect("filter should compose clauses with $and");
        assert_eq!(clauses.len(), 2);

        let tracking_clause = clauses[0]
            .as_document()
            .expect("tracking clause should be a document");
        let seq = tracking_clause
            .get_document("seq")
            .expect("seq filter should be present");
        assert_eq!(seq.get("$gt"), Some(&Bson::Int64(42)));

        let query_clause = clauses[1]
            .as_document()
            .expect("query clause should be a document");
        let query_seq = query_clause
            .get_document("seq")
            .expect("query seq filter should be present");
        assert_eq!(query_seq.get("$lt"), Some(&Bson::Int32(100)));
    }

    #[test]
    fn projection_missing_tracking_field_fails_fast() {
        let doc = doc! {"name": "event_1"};
        let result = extract_tracking_offset_from_document(&doc, "seq");
        assert!(
            matches!(result, Err(Error::InvalidRecord)),
            "Expected InvalidRecord when tracking field is missing"
        );
    }

    #[test]
    fn tracking_field_error_names_collection_field_and_kind() {
        let doc = doc! {"is_processed": true};
        let result = extract_tracking_offset_with_context(&doc, "events", "is_processed");
        match result {
            Err(Error::Storage(message)) => {
                assert!(message.contains("events"));
                assert!(message.contains("is_processed"));
                assert!(message.contains("boolean"));
                assert!(message.contains(SUPPORTED_TRACKING_BSON_KINDS));
            }
            other => panic!("Expected explicit storage error, got {other:?}"),
        }
    }

    #[test]
    fn non_unique_tracking_field_does_not_skip_equal_offsets() {
        let batch_offsets = vec![given_int_tracking_offset(1), given_int_tracking_offset(2)];
        let extra_offset = given_int_tracking_offset(2);
        let checkpoint =
            resolve_checkpoint_offset_for_batch(&batch_offsets, Some(&extra_offset), "seq");
        assert_eq!(
            checkpoint,
            Some(given_int_tracking_offset(1)),
            "Checkpoint should roll back to previous distinct offset at duplicate boundary"
        );
    }

    #[test]
    fn all_equal_duplicate_boundary_should_fail_fast() {
        let batch_offsets = vec![given_int_tracking_offset(7), given_int_tracking_offset(7)];
        let extra_offset = given_int_tracking_offset(7);

        assert!(matches!(
            classify_duplicate_boundary_now(&batch_offsets, Some(&extra_offset), "seq"),
            DuplicateBoundaryOutcome::Fail
        ));
        assert_eq!(
            resolve_checkpoint_offset_for_batch(&batch_offsets, Some(&extra_offset), "seq"),
            None
        );
    }

    #[test]
    fn typed_string_offset_round_trip_preserves_string_kind() {
        let state = State {
            last_poll_time: Utc::now(),
            tracking_offsets: HashMap::from([(
                "test_collection".to_string(),
                PersistedTrackingOffset::Typed(TrackingOffsetValue::String("42".to_string())),
            )]),
            processed_documents: 5,
        };
        let connector_state =
            ConnectorState::serialize(&state, CONNECTOR_NAME, 1).expect("state should serialize");
        let restored = connector_state
            .deserialize::<State>(CONNECTOR_NAME, 1)
            .expect("state should deserialize");

        let restored_offset = restored
            .tracking_offsets
            .get("test_collection")
            .expect("offset should exist");
        assert_eq!(
            restored_offset,
            &PersistedTrackingOffset::Typed(TrackingOffsetValue::String("42".to_string()))
        );
        assert_eq!(
            restored_offset.to_query_bson_now("seq"),
            Bson::String("42".to_string())
        );
    }

    #[test]
    fn legacy_string_offset_remains_backward_compatible() {
        #[derive(Serialize)]
        struct LegacyState {
            last_poll_time: DateTime<Utc>,
            tracking_offsets: HashMap<String, String>,
            processed_documents: u64,
        }

        let connector_state = ConnectorState::serialize(
            &LegacyState {
                last_poll_time: Utc::now(),
                tracking_offsets: HashMap::from([(
                    "test_collection".to_string(),
                    "42".to_string(),
                )]),
                processed_documents: 5,
            },
            CONNECTOR_NAME,
            1,
        )
        .expect("legacy state should serialize");
        let restored = connector_state
            .deserialize::<State>(CONNECTOR_NAME, 1)
            .expect("legacy state should deserialize");

        assert_eq!(
            restored.tracking_offsets.get("test_collection"),
            Some(&PersistedTrackingOffset::LegacyString("42".to_string()))
        );
        assert_eq!(
            restored
                .tracking_offsets
                .get("test_collection")
                .expect("offset should exist")
                .to_query_bson_now("seq"),
            Bson::Int64(42)
        );
    }

    #[tokio::test]
    async fn payload_field_honors_snake_case_conversion() {
        let mut config = given_default_config();
        config.snake_case_fields = Some(true);
        config.payload_field = Some("eventName".to_string());
        let source = MongoDbSource::new(1, config, None);

        let message = source
            .document_to_message(doc! {"eventName": "alpha"}, "seq")
            .await
            .expect("document should convert to message");
        let payload: String =
            serde_json::from_slice(&message.payload).expect("payload should be valid JSON");
        assert_eq!(payload, "alpha");
    }

    #[tokio::test]
    async fn commit_moves_pending_batch_to_committed_state() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);

        {
            let mut state = source.state.lock().await;
            state.pending_batch = Some(PendingBatchState {
                collection: "test_collection".to_string(),
                checkpoint_offset: Some(given_int_tracking_offset(11)),
                processed_count: 3,
            });
        }

        let connector_state = source
            .commit_polled_messages_now()
            .await
            .expect("commit should succeed")
            .expect("state should serialize");
        let restored = connector_state
            .deserialize::<State>(CONNECTOR_NAME, 1)
            .expect("state should deserialize");

        assert_eq!(restored.processed_documents, 3);
        assert_eq!(
            restored.tracking_offsets.get("test_collection"),
            Some(&given_int_tracking_offset(11))
        );

        let live_state = source.state.lock().await;
        assert!(live_state.pending_batch.is_none());
        assert_eq!(live_state.committed_state.processed_documents, 3);
    }

    #[tokio::test]
    async fn discard_clears_pending_batch_without_advancing_state() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);

        {
            let mut state = source.state.lock().await;
            state.committed_state.processed_documents = 5;
            state
                .committed_state
                .tracking_offsets
                .insert("test_collection".to_string(), given_int_tracking_offset(10));
            state.pending_batch = Some(PendingBatchState {
                collection: "test_collection".to_string(),
                checkpoint_offset: Some(given_int_tracking_offset(11)),
                processed_count: 3,
            });
        }

        source
            .discard_polled_messages_now()
            .await
            .expect("discard should succeed");

        let state = source.state.lock().await;
        assert!(state.pending_batch.is_none());
        assert_eq!(state.committed_state.processed_documents, 5);
        assert_eq!(
            state
                .committed_state
                .tracking_offsets
                .get("test_collection"),
            Some(&given_int_tracking_offset(10))
        );
    }
}


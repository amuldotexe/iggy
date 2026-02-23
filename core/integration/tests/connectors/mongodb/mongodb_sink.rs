/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use super::{POLL_ATTEMPTS, POLL_INTERVAL_MS, TEST_MESSAGE_COUNT};
use crate::connectors::fixtures::{
    MongoDbOps, MongoDbSinkAutoCreateFixture, MongoDbSinkBatchFixture, MongoDbSinkFixture,
    MongoDbSinkJsonFixture,
};
use bytes::Bytes;
use iggy::prelude::{IggyMessage, Partitioning};
use iggy_binary_protocol::MessageClient;
use iggy_common::Identifier;
use integration::harness::seeds;
use integration::iggy_harness;
use std::time::Duration;
use tokio::time::sleep;

const DEFAULT_SINK_COLLECTION: &str = "iggy_messages";
const LARGE_BATCH_COUNT: usize = 50;

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/sink.toml")),
    seed = seeds::connector_stream
)]
async fn json_messages_sink_to_mongodb(harness: &TestHarness, fixture: MongoDbSinkJsonFixture) {
    let client = harness.root_client().await.unwrap();
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();

    let json_payloads: Vec<serde_json::Value> = vec![
        serde_json::json!({"name": "Alice", "age": 30}),
        serde_json::json!({"name": "Bob", "score": 99}),
        serde_json::json!({"name": "Carol", "active": true}),
    ];

    let mut messages: Vec<IggyMessage> = json_payloads
        .iter()
        .enumerate()
        .map(|(i, payload)| {
            let bytes = serde_json::to_vec(payload).expect("Failed to serialize");
            IggyMessage::builder()
                .id((i + 1) as u128)
                .payload(Bytes::from(bytes))
                .build()
                .expect("Failed to build message")
        })
        .collect();

    client
        .send_messages(
            &stream_id,
            &topic_id,
            &Partitioning::partition_id(0),
            &mut messages,
        )
        .await
        .expect("Failed to send messages");

    // Wait for connector to consume and insert into MongoDB.
    let docs = fixture
        .wait_for_documents(&mongo_client, DEFAULT_SINK_COLLECTION, TEST_MESSAGE_COUNT)
        .await
        .expect("Documents did not appear in MongoDB");

    assert_eq!(docs.len(), TEST_MESSAGE_COUNT);

    // Verify metadata fields are present on first document.
    let first = &docs[0];
    assert!(
        first.contains_key("iggy_offset"),
        "Expected iggy_offset field"
    );
    assert!(
        first.contains_key("iggy_stream"),
        "Expected iggy_stream field"
    );
    assert!(
        first.contains_key("iggy_topic"),
        "Expected iggy_topic field"
    );
    assert!(
        first.contains_key("iggy_partition_id"),
        "Expected iggy_partition_id field"
    );
    assert!(
        first.contains_key("iggy_timestamp"),
        "Expected iggy_timestamp field"
    );

    // Verify offset sequence is contiguous.
    for (i, doc) in docs.iter().enumerate() {
        let offset = doc.get_i64("iggy_offset").expect("iggy_offset missing");
        assert_eq!(offset, i as i64, "Offset mismatch at document {i}");
    }

    // Verify payload is stored as a BSON Document (queryable) not Binary.
    let payload = first.get("payload").expect("payload field missing");
    assert!(
        matches!(payload, mongodb::bson::Bson::Document(_)),
        "Expected payload to be BSON Document for json format, got: {payload:?}"
    );
}

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/sink.toml")),
    seed = seeds::connector_stream
)]
async fn binary_messages_sink_as_bson_binary(harness: &TestHarness, fixture: MongoDbSinkFixture) {
    let client = harness.root_client().await.unwrap();
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();

    let raw_payloads: Vec<Vec<u8>> = vec![
        b"plain text message".to_vec(),
        vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD],
        vec![0xDE, 0xAD, 0xBE, 0xEF],
    ];

    let mut messages: Vec<IggyMessage> = raw_payloads
        .iter()
        .enumerate()
        .map(|(i, payload)| {
            IggyMessage::builder()
                .id((i + 1) as u128)
                .payload(Bytes::from(payload.clone()))
                .build()
                .expect("Failed to build message")
        })
        .collect();

    client
        .send_messages(
            &stream_id,
            &topic_id,
            &Partitioning::partition_id(0),
            &mut messages,
        )
        .await
        .expect("Failed to send messages");

    let docs = fixture
        .wait_for_documents(&mongo_client, DEFAULT_SINK_COLLECTION, raw_payloads.len())
        .await
        .expect("Documents did not appear");

    assert_eq!(docs.len(), raw_payloads.len());

    for (i, doc) in docs.iter().enumerate() {
        let payload = doc.get("payload").expect("payload field missing");
        match payload {
            mongodb::bson::Bson::Binary(bin) => {
                assert_eq!(
                    bin.subtype,
                    mongodb::bson::spec::BinarySubtype::Generic,
                    "Expected Generic subtype at doc {i}"
                );
                assert_eq!(
                    bin.bytes, raw_payloads[i],
                    "Payload bytes mismatch at doc {i}"
                );
            }
            other => panic!("Expected Binary, got {other:?} at doc {i}"),
        }
    }
}

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/sink.toml")),
    seed = seeds::connector_stream
)]
async fn large_batch_processed_correctly(harness: &TestHarness, fixture: MongoDbSinkBatchFixture) {
    let client = harness.root_client().await.unwrap();
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();

    let mut messages: Vec<IggyMessage> = (0..LARGE_BATCH_COUNT)
        .map(|i| {
            let payload =
                serde_json::to_vec(&serde_json::json!({"idx": i})).expect("Failed to serialize");
            IggyMessage::builder()
                .id((i + 1) as u128)
                .payload(Bytes::from(payload))
                .build()
                .expect("Failed to build message")
        })
        .collect();

    client
        .send_messages(
            &stream_id,
            &topic_id,
            &Partitioning::partition_id(0),
            &mut messages,
        )
        .await
        .expect("Failed to send messages");

    let docs = fixture
        .wait_for_documents(&mongo_client, DEFAULT_SINK_COLLECTION, LARGE_BATCH_COUNT)
        .await
        .expect("Not all documents appeared");

    assert!(
        docs.len() >= LARGE_BATCH_COUNT,
        "Expected at least {LARGE_BATCH_COUNT} documents, got {}",
        docs.len()
    );

    // Verify offsets are contiguous (0..N).
    for (i, doc) in docs.iter().enumerate() {
        let offset = doc.get_i64("iggy_offset").expect("iggy_offset missing");
        assert_eq!(offset, i as i64, "Offset gap detected at position {i}");
    }
}

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/sink.toml")),
    seed = seeds::connector_stream
)]
async fn auto_create_collection_on_open(
    harness: &TestHarness,
    fixture: MongoDbSinkAutoCreateFixture,
) {
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    // The connector's open() creates the collection. Poll until it appears.
    // No messages are sent in this test.
    let mut found = false;
    for _ in 0..POLL_ATTEMPTS {
        if fixture
            .collection_exists(&mongo_client, DEFAULT_SINK_COLLECTION)
            .await
            .unwrap_or(false)
        {
            found = true;
            break;
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert!(
        found,
        "Collection '{DEFAULT_SINK_COLLECTION}' was not created by open() within timeout"
    );

    // No messages sent -- collection should be empty.
    let count = fixture
        .count_documents_in_collection(&mongo_client, DEFAULT_SINK_COLLECTION)
        .await
        .expect("Failed to count");
    assert_eq!(
        count, 0,
        "Collection should be empty after open() with no messages"
    );

    // Suppress unused harness warning.
    let _ = harness;
}

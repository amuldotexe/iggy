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
    MongoDbOps, MongoDbSourceDeleteFixture, MongoDbSourceFixture, MongoDbSourceMarkFixture,
    MongoDbSourceObjectIdDeleteFixture, MongoDbSourceObjectIdFixture,
    MongoDbSourceObjectIdMarkFixture,
};
use iggy_binary_protocol::MessageClient;
use iggy_common::{Consumer, Identifier, PollingStrategy};
use integration::harness::seeds;
use integration::iggy_harness;
use std::time::Duration;
use tokio::time::sleep;

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/source.toml")),
    seed = seeds::connector_stream
)]
async fn source_polls_documents_to_iggy(harness: &TestHarness, fixture: MongoDbSourceFixture) {
    let client = harness.root_client().await.unwrap();
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    // Seed documents before the connector starts polling.
    let docs: Vec<_> = (1..=TEST_MESSAGE_COUNT)
        .map(|i| {
            mongodb::bson::doc! {
                "seq": i as i64,
                "name": format!("event_{i}"),
                "value": (i * 10) as i64,
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, docs)
        .await
        .expect("Failed to seed documents");
    mongo_client.shutdown().await;

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();
    let consumer_id: Identifier = "test_consumer".try_into().unwrap();

    let mut received: Vec<serde_json::Value> = Vec::new();
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            for msg in polled.messages {
                if let Ok(json) = serde_json::from_slice(&msg.payload) {
                    received.push(json);
                }
            }
            if received.len() >= TEST_MESSAGE_COUNT {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert!(
        received.len() >= TEST_MESSAGE_COUNT,
        "Expected at least {TEST_MESSAGE_COUNT} messages from source, got {}",
        received.len()
    );

    // Verify documents arrive in seq order.
    for (i, doc) in received.iter().enumerate() {
        let seq = doc["seq"].as_i64().expect("seq field missing");
        assert_eq!(seq, (i + 1) as i64, "Seq mismatch at position {i}");
    }
}

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/source.toml")),
    seed = seeds::connector_stream
)]
async fn delete_after_read_removes_documents(
    harness: &TestHarness,
    fixture: MongoDbSourceDeleteFixture,
) {
    let client = harness.root_client().await.unwrap();
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    let docs: Vec<_> = (1..=TEST_MESSAGE_COUNT)
        .map(|i| {
            mongodb::bson::doc! {
                "seq": i as i64,
                "name": format!("event_{i}"),
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, docs)
        .await
        .expect("Failed to seed");

    let initial_count = fixture
        .count_documents(&mongo_client)
        .await
        .expect("count failed");
    assert_eq!(initial_count, TEST_MESSAGE_COUNT as u64);

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();
    let consumer_id: Identifier = "test_consumer".try_into().unwrap();

    // Wait for messages to appear in iggy stream.
    let mut received_count = 0usize;
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            received_count += polled.messages.len();
            if received_count >= TEST_MESSAGE_COUNT {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert!(
        received_count >= TEST_MESSAGE_COUNT,
        "Messages not received from source"
    );

    // Wait for delete_after_read to complete.
    let mut final_count = initial_count;
    for _ in 0..POLL_ATTEMPTS {
        final_count = fixture
            .count_documents(&mongo_client)
            .await
            .unwrap_or(initial_count);
        if final_count == 0 {
            break;
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert_eq!(
        final_count, 0,
        "Expected 0 documents after delete_after_read, got {final_count}"
    );

    mongo_client.shutdown().await;
}

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/source.toml")),
    seed = seeds::connector_stream
)]
async fn mark_processed_sets_field(harness: &TestHarness, fixture: MongoDbSourceMarkFixture) {
    let client = harness.root_client().await.unwrap();
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    let docs: Vec<_> = (1..=TEST_MESSAGE_COUNT)
        .map(|i| {
            mongodb::bson::doc! {
                "seq": i as i64,
                "name": format!("event_{i}"),
                "is_processed": false,
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, docs)
        .await
        .expect("Failed to seed");

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();
    let consumer_id: Identifier = "test_consumer".try_into().unwrap();

    let mut received_count = 0usize;
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            received_count += polled.messages.len();
            if received_count >= TEST_MESSAGE_COUNT {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert!(received_count >= TEST_MESSAGE_COUNT);

    // Wait for is_processed to flip on all documents.
    let mut processed = 0u64;
    for _ in 0..POLL_ATTEMPTS {
        processed = fixture
            .count_processed_documents(&mongo_client)
            .await
            .unwrap_or(0);
        if processed >= TEST_MESSAGE_COUNT as u64 {
            break;
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert_eq!(
        processed, TEST_MESSAGE_COUNT as u64,
        "Expected {TEST_MESSAGE_COUNT} processed documents, got {processed}"
    );

    // Total document count unchanged -- no deletes.
    let total = fixture
        .count_documents(&mongo_client)
        .await
        .expect("count failed");
    assert_eq!(
        total, TEST_MESSAGE_COUNT as u64,
        "Documents should not be deleted when using mark-processed"
    );

    mongo_client.shutdown().await;
}

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/source.toml")),
    seed = seeds::connector_stream
)]
async fn state_persists_across_connector_restart(
    harness: &mut TestHarness,
    fixture: MongoDbSourceFixture,
) {
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    // Seed first batch.
    let first_batch: Vec<_> = (1..=TEST_MESSAGE_COUNT)
        .map(|i| {
            mongodb::bson::doc! {
                "seq": i as i64,
                "name": format!("batch1_{i}"),
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, first_batch)
        .await
        .expect("Failed to seed");

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();
    let consumer_id: Identifier = "restart_test_consumer".try_into().unwrap();

    let client = harness.root_client().await.unwrap();

    // Consume first batch.
    let mut received_before = 0usize;
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            received_before += polled.messages.len();
            if received_before >= TEST_MESSAGE_COUNT {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
    assert_eq!(
        received_before, TEST_MESSAGE_COUNT,
        "First batch not fully consumed"
    );

    // Stop connectors runtime.
    harness
        .server_mut()
        .stop_dependents()
        .expect("Failed to stop connectors");

    // Seed second batch while connector is stopped.
    let second_batch_start = TEST_MESSAGE_COUNT + 1;
    let second_batch: Vec<_> = (second_batch_start..=(second_batch_start + TEST_MESSAGE_COUNT - 1))
        .map(|i| {
            mongodb::bson::doc! {
                "seq": i as i64,
                "name": format!("batch2_{i}"),
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, second_batch)
        .await
        .expect("Failed to seed batch 2");

    // Restart connectors.
    harness
        .server_mut()
        .start_dependents()
        .await
        .expect("Failed to restart connectors");
    sleep(Duration::from_secs(2)).await;

    // Consume after restart. Expect only second batch (no duplicates).
    let mut received_after: Vec<serde_json::Value> = Vec::new();
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            for msg in polled.messages {
                if let Ok(json) = serde_json::from_slice(&msg.payload) {
                    received_after.push(json);
                }
            }
            if received_after.len() >= TEST_MESSAGE_COUNT {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert_eq!(
        received_after.len(),
        TEST_MESSAGE_COUNT,
        "Second batch count mismatch"
    );

    // All messages from second poll must have seq > TEST_MESSAGE_COUNT.
    for msg in &received_after {
        let seq = msg["seq"].as_i64().expect("seq field missing");
        assert!(
            seq > TEST_MESSAGE_COUNT as i64,
            "Got seq={seq} from first batch after restart -- duplicate detected"
        );
    }

    mongo_client.shutdown().await;
}

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/source.toml")),
    seed = seeds::connector_stream
)]
async fn source_polls_documents_by_object_id(
    harness: &TestHarness,
    fixture: MongoDbSourceObjectIdFixture,
) {
    let client = harness.root_client().await.unwrap();
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();
    let consumer_id: Identifier = "objectid_consumer".try_into().unwrap();

    // === PHASE 1: First batch - stores ObjectId offset in state ===
    // Without a custom tracking_field, MongoDB auto-generates ObjectId for _id.
    let first_batch: Vec<_> = (1..=3)
        .map(|i| {
            mongodb::bson::doc! {
                "name": format!("batch1_{i}"),
                "value": i as i64,
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, first_batch)
        .await
        .expect("Failed to seed first batch");

    // Poll first batch - this stores ObjectId offset in connector state
    let mut received_first = 0usize;
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            received_first += polled.messages.len();
            if received_first >= 3 {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
    assert_eq!(received_first, 3, "First batch should have 3 documents");

    // === PHASE 2: Second batch - THIS IS WHERE THE BUG WOULD MANIFEST ===
    // Without the fix, the $gt query would use string comparison against ObjectId:
    // { "_id": { "$gt": "507f1f77bcf86cd799439011" } }  // WRONG: string vs ObjectId
    // This FAILS because MongoDB doesn't coerce types!
    sleep(Duration::from_millis(500)).await; // Ensure new ObjectIds are greater

    let second_batch: Vec<_> = (4..=6)
        .map(|i| {
            mongodb::bson::doc! {
                "name": format!("batch2_{i}"),
                "value": i as i64,
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, second_batch)
        .await
        .expect("Failed to seed second batch");

    // Poll second batch - uses $gt with stored ObjectId offset
    // This is the actual bug test: if ObjectId conversion is broken,
    // the $gt filter won't match any documents.
    let mut received_second = 0usize;
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            received_second += polled.messages.len();
            if received_second >= 3 {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
    assert_eq!(
        received_second, 3,
        "Second batch should have 3 documents - ObjectId $gt comparison working"
    );

    mongo_client.shutdown().await;
}

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/source.toml")),
    seed = seeds::connector_stream
)]
async fn source_delete_after_read_with_object_id(
    harness: &TestHarness,
    fixture: MongoDbSourceObjectIdDeleteFixture,
) {
    let client = harness.root_client().await.unwrap();
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();
    let consumer_id: Identifier = "objectid_delete_consumer".try_into().unwrap();

    // === PHASE 1: Seed documents - MongoDB auto-generates ObjectId for _id ===
    let docs: Vec<_> = (1..=TEST_MESSAGE_COUNT)
        .map(|i| {
            mongodb::bson::doc! {
                "name": format!("event_{i}"),
                "value": (i * 10) as i64,
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, docs)
        .await
        .expect("Failed to seed documents");

    // Verify initial count
    let initial_count = fixture
        .count_documents(&mongo_client)
        .await
        .expect("count failed");
    assert_eq!(
        initial_count, TEST_MESSAGE_COUNT as u64,
        "Initial count should match seeded"
    );

    // === PHASE 2: Poll messages - this stores ObjectId offset and triggers delete ===
    let mut received = 0usize;
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            received += polled.messages.len();
            if received >= TEST_MESSAGE_COUNT {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
    assert_eq!(
        received, TEST_MESSAGE_COUNT,
        "Should receive all seeded messages"
    );

    // Wait for delete to process
    sleep(Duration::from_millis(500)).await;

    // === PHASE 3: Verify all documents deleted via $lte ObjectId ===
    // This is the key test: delete_processed_documents uses $lte with ObjectId.
    // If ObjectId conversion is broken, delete would fail silently.
    let remaining = fixture
        .count_documents(&mongo_client)
        .await
        .expect("count failed");
    assert_eq!(
        remaining, 0,
        "All documents should be deleted - ObjectId $lte in delete_processed_documents works"
    );

    mongo_client.shutdown().await;
}

#[iggy_harness(
    server(connectors_runtime(config_path = "tests/connectors/mongodb/source.toml")),
    seed = seeds::connector_stream
)]
async fn source_mark_processed_with_object_id(
    harness: &TestHarness,
    fixture: MongoDbSourceObjectIdMarkFixture,
) {
    let client = harness.root_client().await.unwrap();
    let mongo_client = fixture
        .create_client()
        .await
        .expect("Failed to create MongoDB client");

    let stream_id: Identifier = seeds::names::STREAM.try_into().unwrap();
    let topic_id: Identifier = seeds::names::TOPIC.try_into().unwrap();
    let consumer_id: Identifier = "objectid_mark_consumer".try_into().unwrap();

    // === PHASE 1: First batch - stores ObjectId offset ===
    let first_batch: Vec<_> = (1..=3)
        .map(|i| {
            mongodb::bson::doc! {
                "name": format!("batch1_{i}"),
                "value": i as i64,
                "processed": false,
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, first_batch)
        .await
        .expect("Failed to seed first batch");

    // Poll first batch
    let mut received_first = 0usize;
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            received_first += polled.messages.len();
            if received_first >= 3 {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
    assert_eq!(received_first, 3, "First batch should have 3 documents");

    // Wait for mark to process
    sleep(Duration::from_millis(500)).await;

    // === PHASE 2: Second batch - tests $lte ObjectId in mark_documents_processed ===
    let second_batch: Vec<_> = (4..=6)
        .map(|i| {
            mongodb::bson::doc! {
                "name": format!("batch2_{i}"),
                "value": i as i64,
                "processed": false,
            }
        })
        .collect();
    fixture
        .seed_documents(&mongo_client, second_batch)
        .await
        .expect("Failed to seed second batch");

    // Poll second batch
    let mut received_second = 0usize;
    for _ in 0..POLL_ATTEMPTS {
        if let Ok(polled) = client
            .poll_messages(
                &stream_id,
                &topic_id,
                None,
                &Consumer::new(consumer_id.clone()),
                &PollingStrategy::next(),
                10,
                true,
            )
            .await
        {
            received_second += polled.messages.len();
            if received_second >= 3 {
                break;
            }
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
    assert_eq!(received_second, 3, "Second batch should have 3 documents");

    // Wait for mark to process
    sleep(Duration::from_millis(500)).await;

    // === PHASE 3: Verify first batch marked via $lte ObjectId ===
    // Check that documents are marked as processed (not deleted)
    let total = fixture
        .count_documents(&mongo_client)
        .await
        .expect("count failed");
    assert_eq!(
        total, 6,
        "All 6 documents should still exist with mark-processed mode"
    );

    // === PHASE 3: Verify ALL documents marked processed=true ===
    // Wait for mark to complete on all 6 documents
    let mut processed_count = 0u64;
    for _ in 0..POLL_ATTEMPTS {
        processed_count = fixture
            .count_documents_by_field(&mongo_client, "processed", true)
            .await
            .unwrap_or(0);
        if processed_count >= 6 {
            break;
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert_eq!(
        processed_count, 6,
        "Expected all 6 documents to have processed=true, got {processed_count}"
    );

    // Verify no documents have processed=false
    let unprocessed = fixture
        .count_documents_by_field(&mongo_client, "processed", false)
        .await
        .expect("count failed");
    assert_eq!(
        unprocessed, 0,
        "No documents should have processed=false after mark_documents_processed"
    );

    mongo_client.shutdown().await;
}

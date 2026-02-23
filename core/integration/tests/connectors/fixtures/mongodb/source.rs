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

use super::container::{
    DEFAULT_TEST_DATABASE, DEFAULT_TEST_STREAM, DEFAULT_TEST_TOPIC, ENV_SOURCE_COLLECTION,
    ENV_SOURCE_CONNECTION_URI, ENV_SOURCE_DATABASE, ENV_SOURCE_DELETE_AFTER_READ, ENV_SOURCE_PATH,
    ENV_SOURCE_PAYLOAD_FORMAT, ENV_SOURCE_POLL_INTERVAL, ENV_SOURCE_PROCESSED_FIELD,
    ENV_SOURCE_STREAMS_0_SCHEMA, ENV_SOURCE_STREAMS_0_STREAM, ENV_SOURCE_STREAMS_0_TOPIC,
    ENV_SOURCE_TRACKING_FIELD, MongoDbContainer, MongoDbOps,
};
use async_trait::async_trait;
use integration::harness::{TestBinaryError, TestFixture};
use mongodb::{Client, bson::Document};
use std::collections::HashMap;

const SOURCE_COLLECTION: &str = "test_events";

/// MongoDB source connector fixture (JSON payload format, tracking by integer seq field).
pub struct MongoDbSourceFixture {
    container: MongoDbContainer,
}

impl MongoDbOps for MongoDbSourceFixture {
    fn container(&self) -> &MongoDbContainer {
        &self.container
    }
}

impl MongoDbSourceFixture {
    #[allow(dead_code)]
    pub fn collection_name(&self) -> &str {
        SOURCE_COLLECTION
    }

    /// Insert documents with integer `seq` field used as tracking field.
    pub async fn seed_documents(
        &self,
        client: &Client,
        docs: Vec<Document>,
    ) -> Result<(), TestBinaryError> {
        let db = client.database(DEFAULT_TEST_DATABASE);
        let collection = db.collection::<Document>(SOURCE_COLLECTION);
        collection
            .insert_many(docs)
            .await
            .map_err(|e| TestBinaryError::FixtureSetup {
                fixture_type: "MongoDbSourceFixture".to_string(),
                message: format!("Failed to insert documents: {e}"),
            })?;
        Ok(())
    }

    /// Count all documents in the source collection.
    pub async fn count_documents(&self, client: &Client) -> Result<u64, TestBinaryError> {
        let db = client.database(DEFAULT_TEST_DATABASE);
        let collection = db.collection::<Document>(SOURCE_COLLECTION);
        collection
            .count_documents(mongodb::bson::doc! {})
            .await
            .map_err(|e| TestBinaryError::InvalidState {
                message: format!("Failed to count documents: {e}"),
            })
    }

    /// Count documents where is_processed field is true.
    pub async fn count_processed_documents(&self, client: &Client) -> Result<u64, TestBinaryError> {
        let db = client.database(DEFAULT_TEST_DATABASE);
        let collection = db.collection::<Document>(SOURCE_COLLECTION);
        collection
            .count_documents(mongodb::bson::doc! { "is_processed": true })
            .await
            .map_err(|e| TestBinaryError::InvalidState {
                message: format!("Failed to count processed documents: {e}"),
            })
    }
}

#[async_trait]
impl TestFixture for MongoDbSourceFixture {
    async fn setup() -> Result<Self, TestBinaryError> {
        let container = MongoDbContainer::start().await?;
        Ok(Self { container })
    }

    fn connectors_runtime_envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();
        envs.insert(
            ENV_SOURCE_CONNECTION_URI.to_string(),
            self.container.connection_uri.clone(),
        );
        envs.insert(
            ENV_SOURCE_DATABASE.to_string(),
            DEFAULT_TEST_DATABASE.to_string(),
        );
        envs.insert(
            ENV_SOURCE_COLLECTION.to_string(),
            SOURCE_COLLECTION.to_string(),
        );
        envs.insert(ENV_SOURCE_POLL_INTERVAL.to_string(), "10ms".to_string());
        envs.insert(ENV_SOURCE_TRACKING_FIELD.to_string(), "seq".to_string());
        envs.insert(ENV_SOURCE_PAYLOAD_FORMAT.to_string(), "json".to_string());
        envs.insert(
            ENV_SOURCE_STREAMS_0_STREAM.to_string(),
            DEFAULT_TEST_STREAM.to_string(),
        );
        envs.insert(
            ENV_SOURCE_STREAMS_0_TOPIC.to_string(),
            DEFAULT_TEST_TOPIC.to_string(),
        );
        envs.insert(ENV_SOURCE_STREAMS_0_SCHEMA.to_string(), "json".to_string());
        envs.insert(
            ENV_SOURCE_PATH.to_string(),
            "../../target/debug/libiggy_connector_mongodb_source".to_string(),
        );
        envs
    }
}

/// MongoDB source fixture with delete_after_read enabled.
pub struct MongoDbSourceDeleteFixture {
    inner: MongoDbSourceFixture,
}

impl std::ops::Deref for MongoDbSourceDeleteFixture {
    type Target = MongoDbSourceFixture;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl TestFixture for MongoDbSourceDeleteFixture {
    async fn setup() -> Result<Self, TestBinaryError> {
        let container = MongoDbContainer::start().await?;
        Ok(Self {
            inner: MongoDbSourceFixture { container },
        })
    }

    fn connectors_runtime_envs(&self) -> HashMap<String, String> {
        let mut envs = self.inner.connectors_runtime_envs();
        envs.insert(ENV_SOURCE_DELETE_AFTER_READ.to_string(), "true".to_string());
        envs
    }
}

/// MongoDB source fixture with is_processed field marking.
pub struct MongoDbSourceMarkFixture {
    inner: MongoDbSourceFixture,
}

impl std::ops::Deref for MongoDbSourceMarkFixture {
    type Target = MongoDbSourceFixture;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl TestFixture for MongoDbSourceMarkFixture {
    async fn setup() -> Result<Self, TestBinaryError> {
        let container = MongoDbContainer::start().await?;
        Ok(Self {
            inner: MongoDbSourceFixture { container },
        })
    }

    fn connectors_runtime_envs(&self) -> HashMap<String, String> {
        let mut envs = self.inner.connectors_runtime_envs();
        envs.insert(
            ENV_SOURCE_PROCESSED_FIELD.to_string(),
            "is_processed".to_string(),
        );
        envs
    }
}

/// MongoDB source fixture using default _id tracking (ObjectId).
/// Does NOT set tracking_field, allowing the connector to use the default "_id" field.
pub struct MongoDbSourceObjectIdFixture {
    inner: MongoDbSourceFixture,
}

impl MongoDbOps for MongoDbSourceObjectIdFixture {
    fn container(&self) -> &MongoDbContainer {
        self.inner.container()
    }
}

impl std::ops::Deref for MongoDbSourceObjectIdFixture {
    type Target = MongoDbSourceFixture;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl TestFixture for MongoDbSourceObjectIdFixture {
    async fn setup() -> Result<Self, TestBinaryError> {
        let container = MongoDbContainer::start().await?;
        Ok(Self {
            inner: MongoDbSourceFixture { container },
        })
    }

    fn connectors_runtime_envs(&self) -> HashMap<String, String> {
        let mut envs = self.inner.connectors_runtime_envs();
        // Remove the tracking_field override to use default "_id"
        envs.remove(ENV_SOURCE_TRACKING_FIELD);
        envs
    }
}

/// MongoDB source fixture with ObjectId tracking AND delete_after_read enabled.
/// Tests the $lte ObjectId fix in delete_processed_documents().
pub struct MongoDbSourceObjectIdDeleteFixture {
    inner: MongoDbSourceFixture,
}

impl MongoDbOps for MongoDbSourceObjectIdDeleteFixture {
    fn container(&self) -> &MongoDbContainer {
        self.inner.container()
    }
}

impl std::ops::Deref for MongoDbSourceObjectIdDeleteFixture {
    type Target = MongoDbSourceFixture;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl TestFixture for MongoDbSourceObjectIdDeleteFixture {
    async fn setup() -> Result<Self, TestBinaryError> {
        let container = MongoDbContainer::start().await?;
        Ok(Self {
            inner: MongoDbSourceFixture { container },
        })
    }

    fn connectors_runtime_envs(&self) -> HashMap<String, String> {
        let mut envs = self.inner.connectors_runtime_envs();
        // Remove tracking_field override to use default "_id"
        envs.remove(ENV_SOURCE_TRACKING_FIELD);
        // Enable delete_after_read
        envs.insert(ENV_SOURCE_DELETE_AFTER_READ.to_string(), "true".to_string());
        envs
    }
}

/// MongoDB source fixture with ObjectId tracking AND processed_field marking.
/// Tests the $lte ObjectId fix in mark_documents_processed().
pub struct MongoDbSourceObjectIdMarkFixture {
    inner: MongoDbSourceFixture,
}

impl MongoDbOps for MongoDbSourceObjectIdMarkFixture {
    fn container(&self) -> &MongoDbContainer {
        self.inner.container()
    }
}

impl std::ops::Deref for MongoDbSourceObjectIdMarkFixture {
    type Target = MongoDbSourceFixture;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl TestFixture for MongoDbSourceObjectIdMarkFixture {
    async fn setup() -> Result<Self, TestBinaryError> {
        let container = MongoDbContainer::start().await?;
        Ok(Self {
            inner: MongoDbSourceFixture { container },
        })
    }

    fn connectors_runtime_envs(&self) -> HashMap<String, String> {
        let mut envs = self.inner.connectors_runtime_envs();
        // Remove tracking_field override to use default "_id"
        envs.remove(ENV_SOURCE_TRACKING_FIELD);
        // Enable processed_field marking instead of delete
        envs.insert(
            ENV_SOURCE_PROCESSED_FIELD.to_string(),
            "processed".to_string(),
        );
        envs.remove(ENV_SOURCE_DELETE_AFTER_READ);
        envs
    }
}

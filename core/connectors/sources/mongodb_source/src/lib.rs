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

use async_trait::async_trait;
use futures::TryStreamExt;
use humantime::Duration as HumanDuration;
use iggy_common::{DateTime, Utc};
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

source_connector!(MongoDbSource);

const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_RETRY_DELAY: &str = "1s";
const DEFAULT_POLL_INTERVAL: &str = "10s";
const DEFAULT_BATCH_SIZE: u32 = 1000;
const CONNECTOR_NAME: &str = "MongoDB source";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PayloadFormat {
    #[default]
    Json,
    Bson,
    String,
}

impl PayloadFormat {
    fn from_config(s: Option<&str>) -> Self {
        match s.map(|s| s.to_lowercase()).as_deref() {
            Some("bson") | Some("binary") => PayloadFormat::Bson,
            Some("string") | Some("text") => PayloadFormat::String,
            _ => PayloadFormat::Json,
        }
    }

    fn to_schema(self) -> Schema {
        match self {
            PayloadFormat::Json => Schema::Json,
            PayloadFormat::Bson => Schema::Raw,
            PayloadFormat::String => Schema::Text,
        }
    }
}

fn is_transient_error(error: &str) -> bool {
    let msg = error.to_lowercase();
    msg.contains("timeout")
        || msg.contains("network")
        || msg.contains("connection")
        || msg.contains("pool")
        || msg.contains("server selection")
}

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

/// Converts an offset string to the appropriate BSON type for query comparison.
///
/// # Arguments
/// * `offset` - The offset value as a string (from state/config)
/// * `tracking_field` - The field being used for tracking (e.g., "_id", "seq")
///
/// # Returns
/// - `Bson::Int64` for numeric offsets
/// - `Bson::ObjectId` for 24-char hex strings when tracking_field is "_id"
/// - `Bson::String` for all other strings (fallback)
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

fn extract_tracking_offset_from_document(
    document: &Document,
    tracking_field: &str,
) -> Result<String, Error> {
    let bson_value = document.get(tracking_field).ok_or(Error::InvalidRecord)?;
    let offset = match bson_value {
        Bson::Int32(v) => v.to_string(),
        Bson::Int64(v) => v.to_string(),
        Bson::Double(v) => v.to_string(),
        Bson::String(s) => s.clone(),
        Bson::ObjectId(oid) => oid.to_hex(),
        _ => return Err(Error::InvalidRecord),
    };

    Ok(offset)
}

fn find_previous_distinct_offset(batch_offsets: &[String]) -> Option<String> {
    let last_offset = batch_offsets.last()?;
    batch_offsets
        .iter()
        .rev()
        .skip(1)
        .find(|offset| *offset != last_offset)
        .cloned()
}

fn resolve_checkpoint_offset_for_batch(
    batch_offsets: &[String],
    extra_offset: Option<&str>,
    tracking_field: &str,
) -> Option<String> {
    let max_offset = batch_offsets.last().cloned();

    if tracking_field == "_id" {
        return max_offset;
    }

    let batch_max_offset = max_offset.as_deref()?;
    let Some(extra_offset) = extra_offset else {
        return max_offset;
    };

    if extra_offset != batch_max_offset {
        return max_offset;
    }

    find_previous_distinct_offset(batch_offsets)
}

fn apply_checkpoint_after_side_effect(
    state: &mut State,
    collection: &str,
    checkpoint_offset: Option<String>,
    processed_count: u64,
    side_effect_result: Result<(), Error>,
) -> Result<(), Error> {
    side_effect_result?;

    if let Some(offset) = checkpoint_offset {
        state
            .tracking_offsets
            .insert(collection.to_string(), offset);
        state.processed_documents += processed_count;
    }

    Ok(())
}

#[derive(Debug)]
pub struct MongoDbSource {
    pub id: u32,
    client: Option<Client>,
    config: MongoDbSourceConfig,
    state: Mutex<State>,
    verbose: bool,
    retry_delay: Duration,
    poll_interval: Duration,
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
    last_poll_time: DateTime<Utc>,
    tracking_offsets: HashMap<String, String>,
    processed_documents: u64,
}

impl MongoDbSource {
    pub fn new(id: u32, config: MongoDbSourceConfig, state: Option<ConnectorState>) -> Self {
        let verbose = config.verbose_logging.unwrap_or(false);

        let delay_str = config.retry_delay.as_deref().unwrap_or(DEFAULT_RETRY_DELAY);
        let retry_delay = HumanDuration::from_str(delay_str)
            .map(|duration| duration.into())
            .unwrap_or_else(|_| Duration::from_secs(1));

        let poll_str = config
            .poll_interval
            .as_deref()
            .unwrap_or(DEFAULT_POLL_INTERVAL);
        let poll_interval = HumanDuration::from_str(poll_str)
            .map(|duration| duration.into())
            .unwrap_or_else(|_| Duration::from_secs(10));

        // Restore persisted state or seed from initial_offset when none exists
        let initial_state = state
            .and_then(|s| s.deserialize(CONNECTOR_NAME, id))
            .unwrap_or_else(|| {
                let mut offsets = HashMap::new();
                if let Some(offset) = &config.initial_offset {
                    offsets.insert(config.collection.clone(), offset.clone());
                }
                State {
                    last_poll_time: Utc::now(),
                    tracking_offsets: offsets,
                    processed_documents: 0,
                }
            });

        MongoDbSource {
            id,
            client: None,
            config,
            state: Mutex::new(initial_state),
            verbose,
            retry_delay,
            poll_interval,
        }
    }

    fn get_collection(&self) -> Result<Collection<Document>, Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| Error::InitError("MongoDB client not initialized".to_string()))?;

        Ok(client
            .database(&self.config.database)
            .collection(&self.config.collection))
    }

    fn serialize_state(&self, state: &State) -> Option<ConnectorState> {
        ConnectorState::serialize(state, CONNECTOR_NAME, self.id)
    }

    fn get_max_retries(&self) -> u32 {
        self.config.max_retries.unwrap_or(DEFAULT_MAX_RETRIES)
    }
}

#[async_trait]
impl Source for MongoDbSource {
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

    async fn poll(&self) -> Result<ProducedMessages, Error> {
        let poll_interval = self.poll_interval;
        tokio::time::sleep(poll_interval).await;

        let messages = self.poll_collection().await?;

        let mut state = self.state.lock().await;
        state.last_poll_time = Utc::now();

        if self.verbose {
            info!(
                "MongoDB source connector ID: {} produced {} messages. Total processed: {}",
                self.id,
                messages.len(),
                state.processed_documents
            );
        } else {
            debug!(
                "MongoDB source connector ID: {} produced {} messages. Total processed: {}",
                self.id,
                messages.len(),
                state.processed_documents
            );
        }

        // Derive schema from payload_format config
        let payload_format = PayloadFormat::from_config(self.config.payload_format.as_deref());
        let schema = payload_format.to_schema();
        let persisted_state = self.serialize_state(&state);

        Ok(ProducedMessages {
            schema,
            messages,
            state: persisted_state,
        })
    }

    async fn close(&mut self) -> Result<(), Error> {
        info!("Closing MongoDB source connector with ID: {}", self.id);

        // Client will be dropped automatically
        self.client.take();

        let state = self.state.lock().await;
        info!(
            "MongoDB source connector ID: {} closed. Total documents processed: {}",
            self.id, state.processed_documents
        );
        Ok(())
    }
}

impl MongoDbSource {
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
    async fn poll_collection(&self) -> Result<Vec<ProducedMessage>, Error> {
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
    async fn execute_poll(&self) -> Result<Vec<ProducedMessage>, Error> {
        let collection = self.get_collection()?;

        // Build query filter
        let tracking_field = self.config.tracking_field.as_deref().unwrap_or("_id");

        let state = self.state.lock().await;
        let last_offset = state.tracking_offsets.get(&self.config.collection).cloned();
        drop(state);

        let mut filter = doc! {};

        // Add tracking field filter if we have an offset
        if let Some(offset) = last_offset {
            let offset_bson = convert_offset_value_to_bson(&offset, tracking_field);
            filter.insert(tracking_field, doc! {"$gt": offset_bson});
        }

        // Apply additional query filter if configured
        if let Some(query_filter_str) = &self.config.query_filter {
            let additional_filter: Document =
                serde_json::from_str(query_filter_str).map_err(|_e| Error::InvalidConfig)?;
            for (key, value) in additional_filter {
                filter.insert(key, value);
            }
        }

        // Apply processed field filter if configured
        if let Some(processed_field) = &self.config.processed_field {
            filter.insert(processed_field, false);
        }

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
            let offset = extract_tracking_offset_from_document(&doc, tracking_field)?;
            batch_offsets.push(offset);

            let message = self.document_to_message(doc, tracking_field).await?;
            messages.push(message);
        }

        let extra_offset = extra_document
            .as_ref()
            .map(|doc| extract_tracking_offset_from_document(doc, tracking_field))
            .transpose()?;
        let max_offset = batch_offsets.last().cloned();
        let checkpoint_offset = resolve_checkpoint_offset_for_batch(
            &batch_offsets,
            extra_offset.as_deref(),
            tracking_field,
        );

        if tracking_field != "_id"
            && let (Some(boundary_offset), Some(extra_offset)) =
                (max_offset.as_deref(), extra_offset.as_deref())
            && extra_offset == boundary_offset
        {
            warn!(
                collection = %self.config.collection,
                tracking_field = %tracking_field,
                boundary_offset = %boundary_offset,
                "Detected duplicate tracking value at batch boundary; rolling checkpoint back to avoid skipping equal offsets"
            );
        }

        // Delete or mark documents FIRST (before checkpointing)
        // This ensures we don't checkpoint an offset if mark/delete fails
        // Pass checkpoint_offset directly to avoid reading stale offset from state
        let expected_count = messages.len() as u64;
        let side_effect_result = if self.config.delete_after_read.unwrap_or(false) {
            self.delete_processed_documents(checkpoint_offset.as_deref(), expected_count)
                .await
        } else if let Some(processed_field) = &self.config.processed_field {
            self.mark_documents_processed(
                processed_field,
                checkpoint_offset.as_deref(),
                expected_count,
            )
            .await
        } else {
            Ok(())
        };

        // THEN update state with new offset (only after successful mark/delete)
        let mut state = self.state.lock().await;
        apply_checkpoint_after_side_effect(
            &mut state,
            &self.config.collection,
            checkpoint_offset,
            expected_count,
            side_effect_result,
        )?;

        Ok(messages)
    }

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
            let payload_value = doc.get(payload_field).ok_or(Error::InvalidRecord)?;

            match payload_format {
                PayloadFormat::Json => {
                    serde_json::to_vec(payload_value).map_err(|_| Error::InvalidRecord)?
                }
                PayloadFormat::Bson => {
                    let mut buf = Vec::new();
                    let bson_doc = doc! { payload_field: payload_value.clone() };
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

    /// Build base filter combining tracking offset, query_filter, and processed_field.
    /// This ensures delete/mark operations respect the same filters as poll().
    fn build_base_filter(
        &self,
        last_offset: Option<&str>,
        tracking_field: &str,
    ) -> Result<Document, Error> {
        let mut filter = doc! {};

        // Add tracking field filter if we have an offset
        if let Some(offset) = last_offset {
            let offset_bson = convert_offset_value_to_bson(offset, tracking_field);
            filter.insert(tracking_field, doc! {"$lte": offset_bson});
        }

        // Apply additional query filter if configured (same as poll())
        if let Some(query_filter_str) = &self.config.query_filter {
            let additional_filter: Document =
                serde_json::from_str(query_filter_str).map_err(|_e| Error::InvalidConfig)?;
            for (key, value) in additional_filter {
                filter.insert(key, value);
            }
        }

        Ok(filter)
    }

    async fn delete_processed_documents(
        &self,
        current_offset: Option<&str>,
        expected_count: u64,
    ) -> Result<(), Error> {
        let collection = self.get_collection()?;
        let tracking_field = self.config.tracking_field.as_deref().unwrap_or("_id");

        if let Some(offset) = current_offset {
            // Build filter using shared logic (includes query_filter if configured)
            let delete_filter = self.build_base_filter(Some(offset), tracking_field)?;

            let result = collection.delete_many(delete_filter).await.map_err(|e| {
                Error::Storage(format!("Failed to delete processed documents: {e}"))
            })?;

            match classify_expected_actual_mismatch(expected_count, result.deleted_count) {
                ExpectedActualCountMismatch::None => {
                    debug!(
                        "Deleted {} processed documents up to offset: {}",
                        result.deleted_count, offset
                    );
                }
                ExpectedActualCountMismatch::Partial { expected, actual } => {
                    tracing::warn!(
                        collection = %self.config.collection,
                        expected,
                        actual,
                        offset = %offset,
                        "delete_processed_documents: partial mismatch (deleted fewer documents than expected)"
                    );
                }
                ExpectedActualCountMismatch::Complete { expected } => {
                    tracing::warn!(
                        collection = %self.config.collection,
                        expected,
                        actual = result.deleted_count,
                        offset = %offset,
                        "delete_processed_documents: complete mismatch (expected deletions but got 0)"
                    );
                }
            }
        }

        Ok(())
    }

    async fn mark_documents_processed(
        &self,
        processed_field: &str,
        current_offset: Option<&str>,
        expected_count: u64,
    ) -> Result<(), Error> {
        let collection = self.get_collection()?;
        let tracking_field = self.config.tracking_field.as_deref().unwrap_or("_id");

        if let Some(offset) = current_offset {
            // Build filter using shared logic (includes query_filter if configured)
            let update_filter = self.build_base_filter(Some(offset), tracking_field)?;
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
                        result.matched_count, offset
                    );
                }
                ExpectedActualCountMismatch::Partial { expected, actual } => {
                    tracing::warn!(
                        collection = %self.config.collection,
                        processed_field = %processed_field,
                        expected,
                        actual,
                        offset = %offset,
                        "mark_documents_processed: partial mismatch (matched fewer documents than expected)"
                    );
                }
                ExpectedActualCountMismatch::Complete { expected } => {
                    tracing::warn!(
                        collection = %self.config.collection,
                        processed_field = %processed_field,
                        expected,
                        actual = result.matched_count,
                        offset = %offset,
                        "mark_documents_processed: complete mismatch (expected matches but got 0)"
                    );
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(state.processed_documents, 0);
        assert!(state.tracking_offsets.is_empty());
    }

    #[test]
    fn given_initial_offset_with_no_persisted_state_should_seed_tracking() {
        let mut config = given_default_config();
        config.initial_offset = Some("63f5b2a0c1234567890abcde".to_string());
        let source = MongoDbSource::new(1, config.clone(), None);

        let state = source.state.try_lock().unwrap();
        assert_eq!(
            state.tracking_offsets.get(&config.collection),
            Some(&"63f5b2a0c1234567890abcde".to_string())
        );
    }

    #[test]
    fn given_no_initial_offset_should_start_from_beginning() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config.clone(), None);

        let state = source.state.try_lock().unwrap();
        assert!(!state.tracking_offsets.contains_key(&config.collection));
    }

    #[test]
    fn given_valid_state_should_serialize_to_connector_state() {
        let config = given_default_config();
        let source = MongoDbSource::new(1, config, None);

        let state = source.state.try_lock().unwrap();
        let connector_state = source.serialize_state(&state);

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
            .build_base_filter(Some("42"), "seq")
            .expect("filter should build");

        let seq = filter
            .get_document("seq")
            .expect("seq filter should be present");
        assert_eq!(seq.get("$lte"), Some(&Bson::Int64(42)));
        assert_eq!(
            filter.get("tenant"),
            Some(&Bson::String("alpha".to_string()))
        );
        assert_eq!(filter.get("kind"), Some(&Bson::String("event".to_string())));
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
    fn non_unique_tracking_field_does_not_skip_equal_offsets() {
        let batch_offsets = vec!["1".to_string(), "2".to_string()];
        let checkpoint = resolve_checkpoint_offset_for_batch(&batch_offsets, Some("2"), "seq");
        assert_eq!(
            checkpoint,
            Some("1".to_string()),
            "Checkpoint should roll back to previous distinct offset at duplicate boundary"
        );
    }

    #[test]
    fn mark_or_delete_failure_does_not_advance_checkpoint() {
        let mut state = State {
            last_poll_time: Utc::now(),
            tracking_offsets: HashMap::from([("test_collection".to_string(), "10".to_string())]),
            processed_documents: 5,
        };

        let result = apply_checkpoint_after_side_effect(
            &mut state,
            "test_collection",
            Some("11".to_string()),
            3,
            Err(Error::Storage("forced failure".to_string())),
        );

        assert!(result.is_err(), "Expected mark/delete failure to propagate");
        assert_eq!(
            state.tracking_offsets.get("test_collection"),
            Some(&"10".to_string()),
            "Checkpoint must not advance on failure"
        );
        assert_eq!(
            state.processed_documents, 5,
            "Processed count must not advance on failure"
        );
    }
}

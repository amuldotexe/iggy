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

use integration::harness::TestBinaryError;
use mongodb::{Client, options::ClientOptions};
use testcontainers_modules::testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tracing::info;

const MONGODB_IMAGE: &str = "mongo";
const MONGODB_TAG: &str = "7";
const MONGODB_PORT: u16 = 27017;
const MONGODB_READY_MSG: &str = "Waiting for connections";

pub(super) const DEFAULT_TEST_STREAM: &str = "test_stream";
pub(super) const DEFAULT_TEST_TOPIC: &str = "test_topic";
pub(super) const DEFAULT_TEST_DATABASE: &str = "iggy_test";

// Source env vars
pub(super) const ENV_SOURCE_CONNECTION_URI: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_PLUGIN_CONFIG_CONNECTION_URI";
pub(super) const ENV_SOURCE_DATABASE: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_PLUGIN_CONFIG_DATABASE";
pub(super) const ENV_SOURCE_COLLECTION: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_PLUGIN_CONFIG_COLLECTION";
pub(super) const ENV_SOURCE_POLL_INTERVAL: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_PLUGIN_CONFIG_POLL_INTERVAL";
pub(super) const ENV_SOURCE_TRACKING_FIELD: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_PLUGIN_CONFIG_TRACKING_FIELD";
pub(super) const ENV_SOURCE_DELETE_AFTER_READ: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_PLUGIN_CONFIG_DELETE_AFTER_READ";
pub(super) const ENV_SOURCE_PROCESSED_FIELD: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_PLUGIN_CONFIG_PROCESSED_FIELD";
pub(super) const _ENV_SOURCE_INCLUDE_METADATA: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_PLUGIN_CONFIG_INCLUDE_METADATA";
pub(super) const ENV_SOURCE_PAYLOAD_FORMAT: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_PLUGIN_CONFIG_PAYLOAD_FORMAT";
pub(super) const ENV_SOURCE_STREAMS_0_STREAM: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_STREAMS_0_STREAM";
pub(super) const ENV_SOURCE_STREAMS_0_TOPIC: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_STREAMS_0_TOPIC";
pub(super) const ENV_SOURCE_STREAMS_0_SCHEMA: &str =
    "IGGY_CONNECTORS_SOURCE_MONGODB_STREAMS_0_SCHEMA";
pub(super) const ENV_SOURCE_PATH: &str = "IGGY_CONNECTORS_SOURCE_MONGODB_PATH";

/// Base container management for MongoDB fixtures.
pub struct MongoDbContainer {
    #[allow(dead_code)]
    container: ContainerAsync<GenericImage>,
    pub(super) connection_uri: String,
}

impl MongoDbContainer {
    pub(super) async fn start() -> Result<Self, TestBinaryError> {
        let container = GenericImage::new(MONGODB_IMAGE, MONGODB_TAG)
            .with_exposed_port(MONGODB_PORT.tcp())
            .with_wait_for(WaitFor::message_on_stdout(MONGODB_READY_MSG))
            .with_mapped_port(0, MONGODB_PORT.tcp())
            .start()
            .await
            .map_err(|e| TestBinaryError::FixtureSetup {
                fixture_type: "MongoDbContainer".to_string(),
                message: format!("Failed to start container: {e}"),
            })?;

        info!("Started MongoDB container");

        let mapped_port = container
            .ports()
            .await
            .map_err(|e| TestBinaryError::FixtureSetup {
                fixture_type: "MongoDbContainer".to_string(),
                message: format!("Failed to get ports: {e}"),
            })?
            .map_to_host_port_ipv4(MONGODB_PORT)
            .ok_or_else(|| TestBinaryError::FixtureSetup {
                fixture_type: "MongoDbContainer".to_string(),
                message: "No mapping for MongoDB port".to_string(),
            })?;

        // Standalone mode: plain URI. No ?directConnection=true needed
        // (directConnection is only required for single-node replica sets).
        let connection_uri = format!("mongodb://localhost:{mapped_port}");

        info!("MongoDB container available at {connection_uri}");

        Ok(Self {
            container,
            connection_uri,
        })
    }

    pub async fn create_client(&self) -> Result<Client, TestBinaryError> {
        let options = ClientOptions::parse(&self.connection_uri)
            .await
            .map_err(|e| TestBinaryError::FixtureSetup {
                fixture_type: "MongoDbContainer".to_string(),
                message: format!("Failed to parse URI: {e}"),
            })?;

        Client::with_options(options).map_err(|e| TestBinaryError::FixtureSetup {
            fixture_type: "MongoDbContainer".to_string(),
            message: format!("Failed to create client: {e}"),
        })
    }
}

/// Common MongoDB operations for fixtures.
pub trait MongoDbOps: Sync {
    fn container(&self) -> &MongoDbContainer;

    fn create_client(
        &self,
    ) -> impl std::future::Future<Output = Result<Client, TestBinaryError>> + Send {
        self.container().create_client()
    }
}

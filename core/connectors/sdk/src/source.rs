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

use crate::log::{CallbackLayer, LogCallback};
use crate::{ConnectorState, Error, Source, get_runtime};
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex, mpsc};
use tokio::{
    sync::{oneshot, watch},
    task::JoinHandle,
};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

#[repr(C)]
pub struct RawMessage {
    pub offset: u64,
    pub headers_ptr: *const u8,
    pub headers_len: usize,
    pub payload_ptr: *const u8,
    pub payload_len: usize,
}

pub type HandleCallback = extern "C" fn(plugin_id: u32, callback: SendCallback) -> i32;

pub type SendCallback = extern "C" fn(plugin_id: u32, messages_ptr: *const u8, messages_len: usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BatchDeliveryCommand {
    Commit,
    Discard,
}

#[derive(Debug)]
struct BatchDeliveryRequest {
    command: BatchDeliveryCommand,
    response_sender: mpsc::Sender<Result<Option<ConnectorState>, Error>>,
}

#[derive(Debug, Default)]
struct BatchDeliverySignalSlot {
    sender: Mutex<Option<oneshot::Sender<BatchDeliveryRequest>>>,
}

impl BatchDeliverySignalSlot {
    fn register_pending_delivery_now(
        &self,
    ) -> Result<oneshot::Receiver<BatchDeliveryRequest>, Error> {
        let (sender, receiver) = oneshot::channel();
        let mut slot = self.sender.lock().map_err(|_| Error::InvalidState)?;
        if slot.is_some() {
            return Err(Error::InvalidState);
        }

        *slot = Some(sender);
        Ok(receiver)
    }

    fn resolve_pending_delivery_now(&self, request: BatchDeliveryRequest) -> Result<(), Error> {
        let mut slot = self.sender.lock().map_err(|_| Error::InvalidState)?;
        let sender = slot.take().ok_or(Error::InvalidState)?;
        sender.send(request).map_err(|_| Error::InvalidState)
    }

    fn clear_pending_delivery_now(&self) {
        if let Ok(mut slot) = self.sender.lock() {
            slot.take();
        }
    }
}

#[derive(Debug)]
pub struct SourceContainer<T: Source + std::fmt::Debug> {
    id: u32,
    source: Option<Arc<T>>,
    shutdown: Option<watch::Sender<()>>,
    task: Option<JoinHandle<()>>,
    delivery_signal: Arc<BatchDeliverySignalSlot>,
}

impl<T: Source + std::fmt::Debug + 'static> SourceContainer<T> {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            source: None,
            shutdown: None,
            task: None,
            delivery_signal: Arc::new(BatchDeliverySignalSlot {
                sender: Mutex::new(None),
            }),
        }
    }

    /// # Safety
    /// Do not copy the configuration pointer
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn open<F, C>(
        &mut self,
        id: u32,
        config_ptr: *const u8,
        config_len: usize,
        state_ptr: *const u8,
        state_len: usize,
        log_callback: LogCallback,
        factory: F,
    ) -> i32
    where
        F: FnOnce(u32, C, Option<ConnectorState>) -> T,
        C: DeserializeOwned,
    {
        unsafe {
            _ = Registry::default()
                .with(CallbackLayer::new(log_callback))
                .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO")))
                .try_init();
            let slice = std::slice::from_raw_parts(config_ptr, config_len);
            let Ok(config_str) = std::str::from_utf8(slice) else {
                error!("Failed to read configuration for source connector with ID: {id}");
                return -1;
            };

            let Ok(config) = serde_json::from_str(config_str) else {
                error!("Failed to parse configuration for source connector with ID: {id}");
                return -1;
            };

            let state = if state_ptr.is_null() {
                None
            } else {
                let state = std::slice::from_raw_parts(state_ptr, state_len);
                let state = ConnectorState(state.to_vec());
                Some(state)
            };

            let mut source = factory(id, config, state);
            let runtime = get_runtime();
            let result = runtime.block_on(source.open());
            self.id = id;
            self.source = Some(Arc::new(source));
            if result.is_ok() { 0 } else { 1 }
        }
    }

    /// # Safety
    /// This is safe to invoke
    pub unsafe fn close(&mut self) -> i32 {
        let Some(source) = self.source.take() else {
            error!(
                "Source connector with ID: {} is not initialized - cannot close.",
                self.id
            );
            return -1;
        };

        info!("Closing source connector with ID: {}...", self.id);
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }

        let runtime = get_runtime();
        if let Some(handle) = self.task.take() {
            let _ = runtime.block_on(handle);
        }

        let Ok(mut source) = Arc::try_unwrap(source) else {
            error!("Source connector with ID: {} was already closed.", self.id);
            return -1;
        };

        runtime.block_on(async {
            if let Err(err) = source.close().await {
                error!(
                    "Failed to close source connector with ID: {}. {err}",
                    self.id
                );
            }
        });
        info!("Closed source connector with ID: {}", self.id);
        0
    }

    /// # Safety
    /// Do not copy the pointer to the messages.
    pub unsafe fn handle(&mut self, callback: SendCallback) -> i32 {
        let Some(source) = self.source.as_ref() else {
            error!(
                "Source connector with ID: {} is not initialized - cannot handle.",
                self.id
            );
            return -1;
        };

        let runtime = get_runtime();
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        let plugin_id = self.id;
        let source = Arc::clone(source);
        let delivery_signal = Arc::clone(&self.delivery_signal);
        let handle = runtime.spawn(async move {
            let _ =
                handle_messages(plugin_id, source, callback, shutdown_rx, delivery_signal).await;
        });

        self.shutdown = Some(shutdown_tx);
        self.task = Some(handle);
        0
    }

    /// # Safety
    /// The output pointers must be valid for writes when provided.
    pub unsafe fn commit(&self, state_ptr: *mut *const u8, state_len: *mut usize) -> i32 {
        if self.source.is_none() {
            error!(
                "Source connector with ID: {} is not initialized - cannot commit.",
                self.id
            );
            return -1;
        };

        let (response_sender, response_receiver) = mpsc::channel();

        if let Err(err) = self
            .delivery_signal
            .resolve_pending_delivery_now(BatchDeliveryRequest {
                command: BatchDeliveryCommand::Commit,
                response_sender,
            })
        {
            error!(
                "Failed to resolve committed delivery for source connector with ID: {}. {err}",
                self.id
            );
            return -1;
        }

        let state = match response_receiver.recv() {
            Ok(Ok(state)) => state,
            Ok(Err(err)) => {
                error!(
                    "Failed to commit polled messages for source connector with ID: {}. {err}",
                    self.id
                );
                return -1;
            }
            Err(_) => {
                error!(
                    "Commit response channel closed for source connector with ID: {}.",
                    self.id
                );
                return -1;
            }
        };

        unsafe {
            if !state_ptr.is_null() && !state_len.is_null() {
                if let Some(state) = state {
                    let mut bytes = state.0.into_boxed_slice();
                    let len = bytes.len();
                    let ptr = bytes.as_mut_ptr();
                    std::mem::forget(bytes);
                    *state_ptr = ptr.cast_const();
                    *state_len = len;
                } else {
                    *state_ptr = std::ptr::null();
                    *state_len = 0;
                }
            }
        }

        0
    }

    /// # Safety
    /// This is safe to invoke.
    pub unsafe fn discard(&self) -> i32 {
        if self.source.is_none() {
            error!(
                "Source connector with ID: {} is not initialized - cannot discard.",
                self.id
            );
            return -1;
        };

        let (response_sender, response_receiver) = mpsc::channel();

        if let Err(err) = self
            .delivery_signal
            .resolve_pending_delivery_now(BatchDeliveryRequest {
                command: BatchDeliveryCommand::Discard,
                response_sender,
            })
        {
            error!(
                "Failed to resolve discarded delivery for source connector with ID: {}. {err}",
                self.id
            );
            return -1;
        }

        match response_receiver.recv() {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => {
                error!(
                    "Failed to discard polled messages for source connector with ID: {}. {err}",
                    self.id
                );
                return -1;
            }
            Err(_) => {
                error!(
                    "Discard response channel closed for source connector with ID: {}.",
                    self.id
                );
                return -1;
            }
        }

        0
    }
}

async fn handle_messages<T: Source>(
    plugin_id: u32,
    source: Arc<T>,
    callback: SendCallback,
    mut shutdown: watch::Receiver<()>,
    delivery_signal: Arc<BatchDeliverySignalSlot>,
) -> Result<(), Error> {
    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                info!("Shutting down source connector with ID: {plugin_id}");
                break;
            }
            messages = source.poll() => {
                let Ok(messages) = messages else {
                    error!("Failed to poll messages for source connector with ID: {plugin_id}");
                    continue;
                };

                let Ok(messages) = postcard::to_allocvec(&messages) else {
                    error!("Failed to serialize messages for source connector with ID: {plugin_id}");
                    continue;
                };

                let Ok(delivery_receiver) = delivery_signal.register_pending_delivery_now() else {
                    error!("Failed to register pending delivery for source connector with ID: {plugin_id}");
                    continue;
                };

                callback(plugin_id, messages.as_ptr(), messages.len());
                tokio::select! {
                    _ = shutdown.changed() => {
                        let _ = source.discard_polled_messages_now().await;
                        delivery_signal.clear_pending_delivery_now();
                        info!("Shutting down source connector with ID: {plugin_id}");
                        break;
                    }
                    request = delivery_receiver => {
                        let Ok(request) = request else {
                            error!("Delivery acknowledgement channel closed for source connector with ID: {plugin_id}");
                            break;
                        };

                        let response = match request.command {
                            BatchDeliveryCommand::Commit => source.commit_polled_messages_now().await,
                            BatchDeliveryCommand::Discard => {
                                source.discard_polled_messages_now().await.map(|_| None)
                            }
                        };

                        if request.response_sender.send(response).is_err() {
                            error!("Failed to return delivery acknowledgement result for source connector with ID: {plugin_id}");
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[macro_export]
macro_rules! source_connector {
    ($type:ty) => {
        const _: fn() = || {
            fn assert_trait<T: $crate::Source>() {}
            assert_trait::<$type>();
        };

        use dashmap::DashMap;
        use once_cell::sync::Lazy;
        use $crate::LogCallback;
        use $crate::source::SendCallback;
        use $crate::source::SourceContainer;

        static INSTANCES: Lazy<DashMap<u32, SourceContainer<$type>>> = Lazy::new(DashMap::new);

        #[cfg(not(test))]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn iggy_source_open(
            id: u32,
            config_ptr: *const u8,
            config_len: usize,
            state_ptr: *const u8,
            state_len: usize,
            log_callback: LogCallback,
        ) -> i32 {
            let mut container = SourceContainer::new(id);
            let result = container.open(
                id,
                config_ptr,
                config_len,
                state_ptr,
                state_len,
                log_callback,
                <$type>::new,
            );
            INSTANCES.insert(id, container);
            result
        }

        #[cfg(not(test))]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn iggy_source_handle(id: u32, callback: SendCallback) -> i32 {
            let Some(mut instance) = INSTANCES.get_mut(&id) else {
                tracing::error!(
                    "Source connector with ID: {id} was not found and cannot be handled."
                );
                return -1;
            };
            instance.handle(callback)
        }

        #[cfg(not(test))]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn iggy_source_close(id: u32) -> i32 {
            let Some(mut instance) = INSTANCES.remove(&id) else {
                tracing::error!(
                    "Source connector with ID: {id} was not found and cannot be closed."
                );
                return -1;
            };
            instance.1.close()
        }

        #[cfg(not(test))]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn iggy_source_commit(
            id: u32,
            state_ptr: *mut *const u8,
            state_len: *mut usize,
        ) -> i32 {
            let Some(instance) = INSTANCES.get(&id) else {
                tracing::error!(
                    "Source connector with ID: {id} was not found and cannot be committed."
                );
                return -1;
            };
            instance.commit(state_ptr, state_len)
        }

        #[cfg(not(test))]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn iggy_source_discard(id: u32) -> i32 {
            let Some(instance) = INSTANCES.get(&id) else {
                tracing::error!(
                    "Source connector with ID: {id} was not found and cannot be discarded."
                );
                return -1;
            };
            instance.discard()
        }

        #[cfg(not(test))]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn iggy_source_free_state(state_ptr: *mut u8, state_len: usize) {
            if !state_ptr.is_null() && state_len > 0 {
                drop(Vec::from_raw_parts(state_ptr, state_len, state_len));
            }
        }

        #[cfg(not(test))]
        #[unsafe(no_mangle)]
        extern "C" fn iggy_source_version() -> *const std::ffi::c_char {
            static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
            VERSION.as_ptr() as *const std::ffi::c_char
        }
    };
}

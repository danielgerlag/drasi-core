// Copyright 2025 The Drasi Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The companion source that exposes each entity's current state as a graph node.
//!
//! The source receives [`SourceChange`]s pushed by the paired reaction (via the
//! [`crate::registry`]) and dispatches them to subscribers. Late-joining
//! subscribers are bootstrapped by reading the persisted entity records from the
//! state store, so the source is fully self-describing and survives restarts.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info, warn};
use tokio::sync::RwLock;
use tracing::Instrument;

use drasi_core::models::{
    Element, ElementMetadata, ElementPropertyMap, ElementReference, SourceChange,
};
use drasi_lib::bootstrap::{
    BootstrapContext, BootstrapProvider, BootstrapRequest, BootstrapResult,
};
use drasi_lib::channels::{
    BootstrapEvent, BootstrapEventSender, ComponentStatus, SourceEvent, SourceEventWrapper,
    SubscriptionResponse,
};
use drasi_lib::context::SourceRuntimeContext;
use drasi_lib::sources::base::{SourceBase, SourceBaseParams};
use drasi_lib::state_store::StateStoreProvider;
use drasi_lib::Source;

use crate::engine::EntityRecord;

/// The source half of a state machine component.
pub struct StateMachineSource {
    base: SourceBase,
    /// Shared state store handle, populated from the runtime context during
    /// `initialize()` and shared with the bootstrap provider.
    state_store: Arc<RwLock<Option<Arc<dyn StateStoreProvider>>>>,
}

impl StateMachineSource {
    /// Create a new state machine source registered under `source_id`.
    pub fn new(source_id: impl Into<String>) -> Result<Self> {
        Self::create(source_id, true)
    }

    /// Create a new state machine source with explicit auto-start behavior.
    pub fn create(source_id: impl Into<String>, auto_start: bool) -> Result<Self> {
        let params = SourceBaseParams::new(source_id.into()).with_auto_start(auto_start);
        Ok(Self {
            base: SourceBase::new(params)?,
            state_store: Arc::new(RwLock::new(None)),
        })
    }

    async fn run_dispatch_loop(&self) -> Result<()> {
        let mut rx = crate::registry::register_source(&self.base.id);
        let source_id = self.base.id.clone();
        let dispatchers = self.base.dispatchers.clone();
        let reporter = self.base.status_handle();

        let instance_id = self
            .base
            .context()
            .await
            .map(|c| c.instance_id)
            .unwrap_or_default();

        let span = tracing::info_span!(
            "state_machine_source",
            instance_id = %instance_id,
            component_id = %source_id,
            component_type = "source"
        );

        let handle = tokio::spawn(
            async move {
                info!("StateMachineSource '{source_id}' dispatch loop started");
                reporter
                    .set_status(
                        ComponentStatus::Running,
                        Some("Exposing entity state".to_string()),
                    )
                    .await;

                while let Some(change) = rx.recv().await {
                    let wrapper = SourceEventWrapper::new(
                        source_id.clone(),
                        SourceEvent::Change(change),
                        chrono::Utc::now(),
                    );
                    if let Err(e) =
                        SourceBase::dispatch_from_task(dispatchers.clone(), wrapper, &source_id)
                            .await
                    {
                        debug!(
                            "[{source_id}] Failed to dispatch state change (no subscribers): {e}"
                        );
                    }
                }

                info!("StateMachineSource '{source_id}' dispatch loop stopped");
            }
            .instrument(span),
        );

        *self.base.task_handle.write().await = Some(handle);
        Ok(())
    }
}

#[async_trait]
impl Source for StateMachineSource {
    fn id(&self) -> &str {
        &self.base.id
    }

    fn type_name(&self) -> &str {
        "state-machine"
    }

    fn properties(&self) -> HashMap<String, serde_json::Value> {
        // The source is self-describing (entity records carry their own label and
        // key field), so it requires no persisted configuration.
        HashMap::new()
    }

    fn auto_start(&self) -> bool {
        self.base.get_auto_start()
    }

    fn supports_replay(&self) -> bool {
        false
    }

    async fn start(&self) -> Result<()> {
        info!("Starting StateMachineSource '{}'", self.base.id);
        self.base
            .set_status(
                ComponentStatus::Starting,
                Some("Starting state machine source".to_string()),
            )
            .await;

        // Install the bootstrap provider that replays persisted entity records.
        let provider = StateMachineBootstrapProvider {
            partition: self.base.id.clone(),
            state_store: self.state_store.clone(),
        };
        self.base.set_bootstrap_provider(provider).await;

        self.run_dispatch_loop().await?;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping StateMachineSource '{}'", self.base.id);
        self.base
            .set_status(
                ComponentStatus::Stopping,
                Some("Stopping state machine source".to_string()),
            )
            .await;

        crate::registry::unregister_source(&self.base.id);
        if let Some(handle) = self.base.task_handle.write().await.take() {
            handle.abort();
        }

        self.base
            .set_status(
                ComponentStatus::Stopped,
                Some("State machine source stopped".to_string()),
            )
            .await;
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        self.base.get_status().await
    }

    async fn subscribe(
        &self,
        settings: drasi_lib::config::SourceSubscriptionSettings,
    ) -> Result<SubscriptionResponse> {
        self.base
            .subscribe_with_bootstrap(&settings, "StateMachine")
            .await
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn initialize(&self, context: SourceRuntimeContext) {
        if let Some(store) = context.state_store.clone() {
            *self.state_store.write().await = Some(store);
        }
        self.base.initialize(context).await;
    }
}

/// Bootstrap provider that replays persisted entity records as node inserts.
struct StateMachineBootstrapProvider {
    partition: String,
    state_store: Arc<RwLock<Option<Arc<dyn StateStoreProvider>>>>,
}

#[async_trait]
impl BootstrapProvider for StateMachineBootstrapProvider {
    async fn bootstrap(
        &self,
        request: BootstrapRequest,
        context: &BootstrapContext,
        event_tx: BootstrapEventSender,
        _settings: Option<&drasi_lib::config::SourceSubscriptionSettings>,
    ) -> Result<BootstrapResult> {
        let store = match self.state_store.read().await.clone() {
            Some(store) => store,
            None => {
                warn!(
                    "StateMachineSource '{}' bootstrap requested but no state store configured",
                    self.partition
                );
                return Ok(BootstrapResult::default());
            }
        };

        let keys = store
            .list_keys(&self.partition)
            .await
            .map_err(|e| anyhow::anyhow!("failed to list entity keys: {e}"))?;

        let mut count = 0usize;
        for key in keys {
            let Some(bytes) = store
                .get(&self.partition, &key)
                .await
                .map_err(|e| anyhow::anyhow!("failed to read entity '{key}': {e}"))?
            else {
                continue;
            };
            let record: EntityRecord = match serde_json::from_slice(&bytes) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Skipping corrupt entity record '{key}': {e}");
                    continue;
                }
            };

            // Filter by requested node labels (all our nodes share one label).
            if !request.node_labels.is_empty() && !request.node_labels.contains(&record.label) {
                continue;
            }

            let element = record_to_node_element(&self.partition, &record);
            let sequence = context.next_sequence();
            event_tx
                .send(BootstrapEvent {
                    source_id: context.source_id.clone(),
                    change: SourceChange::Insert { element },
                    timestamp: chrono::Utc::now(),
                    sequence,
                })
                .await
                .map_err(|e| anyhow::anyhow!("failed to send bootstrap node: {e}"))?;
            count += 1;
        }

        info!(
            "StateMachineSource '{}' bootstrapped {} entity nodes for query '{}'",
            self.partition, count, request.query_id
        );

        Ok(BootstrapResult {
            event_count: count,
            source_position: None,
        })
    }
}

/// Build a graph node [`Element`] from a persisted entity record.
pub(crate) fn record_to_node_element(source_id: &str, record: &EntityRecord) -> Element {
    let properties = ElementPropertyMap::from(&record.node_properties());
    let effective_from = if record.entered_at > 0 {
        record.entered_at as u64
    } else {
        chrono::Utc::now().timestamp_millis().max(0) as u64
    };
    Element::Node {
        metadata: ElementMetadata {
            reference: ElementReference::new(source_id, &record.key),
            labels: Arc::from(vec![Arc::from(record.label.as_str())]),
            effective_from,
        },
        properties,
    }
}

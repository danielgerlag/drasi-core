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

//! The state machine reaction.
//!
//! Consumes results from its input queries, evaluates state transitions via the
//! [`StateMachine`] engine, durably persists each entity's state, and pushes a
//! graph node change to the companion [`StateMachineSource`](crate::StateMachineSource)
//! for downstream queries to observe.

use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use drasi_core::models::SourceChange;
use log::{debug, error, info};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use drasi_lib::channels::{ComponentStatus, QueryResult};
use drasi_lib::context::ReactionRuntimeContext;
use drasi_lib::reactions::common::base::{ReactionBase, ReactionBaseParams};
use drasi_lib::state_store::StateStoreProvider;
use drasi_lib::Reaction;

use crate::config::StateMachineReactionConfig;
use crate::engine::{EntityRecord, StateMachine};
use crate::source::record_to_node_element;

/// The reaction half of a state machine component.
pub struct StateMachineReaction {
    base: ReactionBase,
    config: StateMachineReactionConfig,
    engine: Arc<Mutex<StateMachine>>,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl std::fmt::Debug for StateMachineReaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachineReaction")
            .field("id", &self.base.id)
            .field("config", &self.config)
            .finish()
    }
}

impl StateMachineReaction {
    /// Create a new state machine reaction.
    pub fn new(id: impl Into<String>, config: StateMachineReactionConfig) -> Result<Self> {
        Self::create(id.into(), config, None, true)
    }

    pub(crate) fn create(
        id: String,
        config: StateMachineReactionConfig,
        priority_queue_capacity: Option<usize>,
        auto_start: bool,
    ) -> Result<Self> {
        config.validate()?;

        let queries = config.referenced_queries();
        let mut params = ReactionBaseParams::new(id, queries).with_auto_start(auto_start);
        if let Some(capacity) = priority_queue_capacity {
            params = params.with_priority_queue_capacity(capacity);
        }

        let engine = StateMachine::new(&config)?;

        Ok(Self {
            base: ReactionBase::new(params),
            config,
            engine: Arc::new(Mutex::new(engine)),
            task_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// The id of the companion source this reaction feeds.
    pub fn source_id(&self) -> &str {
        &self.config.source_id
    }

    /// Load persisted entity records into the engine so `previous` guards work
    /// across restarts.
    async fn load_persisted_state(&self, store: &Arc<dyn StateStoreProvider>) -> Result<()> {
        let partition = &self.config.source_id;
        let keys = store
            .list_keys(partition)
            .await
            .map_err(|e| anyhow::anyhow!("failed to list persisted entities: {e}"))?;

        let mut records = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(bytes) = store
                .get(partition, &key)
                .await
                .map_err(|e| anyhow::anyhow!("failed to read entity '{key}': {e}"))?
            {
                match serde_json::from_slice::<EntityRecord>(&bytes) {
                    Ok(record) => records.push(record),
                    Err(e) => error!("[{partition}] skipping corrupt entity record '{key}': {e}"),
                }
            }
        }

        let loaded = records.len();
        self.engine.lock().await.load(records);
        info!(
            "[{}] loaded {} persisted entity states",
            self.base.id, loaded
        );
        Ok(())
    }

    fn spawn_processing_task(&self, store: Arc<dyn StateStoreProvider>) -> JoinHandle<()> {
        let priority_queue = self.base.priority_queue.clone();
        let engine = self.engine.clone();
        let reaction_id = self.base.id.clone();
        let source_id = self.config.source_id.clone();

        tokio::spawn(async move {
            info!("[{reaction_id}] state machine processing loop started");
            loop {
                let query_result_arc = priority_queue.dequeue().await;
                let query_result = (*query_result_arc).clone();
                debug!(
                    "[{reaction_id}] processing result from query '{}'",
                    query_result.query_id
                );

                let transitions = {
                    let mut sm = engine.lock().await;
                    sm.process(&query_result)
                };

                for record in transitions {
                    if let Err(e) = persist_record(store.as_ref(), &source_id, &record).await {
                        error!(
                            "[{reaction_id}] failed to persist entity '{}': {e}",
                            record.key
                        );
                        continue;
                    }

                    let element = record_to_node_element(&source_id, &record);
                    let change = if record.previous_state.is_none() {
                        SourceChange::Insert { element }
                    } else {
                        SourceChange::Update { element }
                    };

                    match crate::registry::current_sender(&source_id) {
                        Some(tx) => {
                            if let Err(e) = tx.send(change).await {
                                debug!(
                                    "[{reaction_id}] could not push state for '{}' (source not receiving): {e}",
                                    record.key
                                );
                            } else {
                                info!(
                                    "[{reaction_id}] entity '{}' -> {} (was {:?})",
                                    record.key, record.state, record.previous_state
                                );
                            }
                        }
                        None => {
                            debug!(
                                "[{reaction_id}] source '{source_id}' not started; '{}' state persisted only",
                                record.key
                            );
                        }
                    }
                }
            }
        })
    }
}

/// Persist a single entity record to the state store, partitioned by source id.
async fn persist_record(
    store: &dyn StateStoreProvider,
    source_id: &str,
    record: &EntityRecord,
) -> Result<()> {
    let bytes = serde_json::to_vec(record).context("serialize entity record")?;
    store
        .set(source_id, &record.key, bytes)
        .await
        .map_err(|e| anyhow::anyhow!("state store set failed: {e}"))
}

#[async_trait]
impl Reaction for StateMachineReaction {
    fn id(&self) -> &str {
        &self.base.id
    }

    fn type_name(&self) -> &str {
        "state-machine"
    }

    fn properties(&self) -> std::collections::HashMap<String, serde_json::Value> {
        self.base.properties_or_serialize(&self.config)
    }

    fn query_ids(&self) -> Vec<String> {
        self.base.queries.clone()
    }

    fn auto_start(&self) -> bool {
        self.base.get_auto_start()
    }

    async fn initialize(&self, context: ReactionRuntimeContext) {
        self.base.initialize(context).await;
    }

    async fn start(&self) -> Result<()> {
        info!("[{}] starting state machine reaction", self.base.id);
        self.base
            .set_status(
                ComponentStatus::Starting,
                Some("Starting state machine".to_string()),
            )
            .await;

        let store = self.base.state_store().await.ok_or_else(|| {
            anyhow::anyhow!(
                "state machine reaction '{}' requires a durable state store, but none is configured",
                self.base.id
            )
        })?;

        self.load_persisted_state(&store).await?;

        let task = self.spawn_processing_task(store);
        *self.task_handle.lock().await = Some(task);

        self.base
            .set_status(ComponentStatus::Running, Some("Running".to_string()))
            .await;
        info!("[{}] state machine reaction started", self.base.id);
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("[{}] stopping state machine reaction", self.base.id);
        if let Some(handle) = self.task_handle.lock().await.take() {
            handle.abort();
        }
        self.base.stop_common().await?;
        self.base
            .set_status(ComponentStatus::Stopped, Some("Stopped".to_string()))
            .await;
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        self.base.get_status().await
    }

    async fn enqueue_query_result(&self, result: QueryResult) -> Result<()> {
        self.base.enqueue_query_result(result).await
    }

    fn is_durable(&self) -> bool {
        true
    }
}

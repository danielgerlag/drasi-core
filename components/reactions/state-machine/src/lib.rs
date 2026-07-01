#![allow(unexpected_cfgs)]
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

//! # State Machine reaction for Drasi
//!
//! A unique Drasi component that is **both a reaction and a source**. It maps live
//! continuous-query results to per-entity state transitions and exposes the
//! realtime state of every entity as a graph source that downstream queries can
//! subscribe to.
//!
//! ## Architecture
//!
//! The component is a paired [`StateMachineReaction`] + [`StateMachineSource`]:
//!
//! * The **reaction** subscribes to the input queries referenced by its states.
//!   Each [`config::EnterCondition`] declares a query, the result `ops` that
//!   trigger it, the allowed `previous` states, and a Handlebars `key` template
//!   that extracts the entity key from the result row. When a result matches and
//!   the entity is in an allowed prior state, the entity transitions; the new
//!   state is persisted to the state store and pushed to the companion source.
//! * The **source** is registered under a configurable `source_id`. It emits one
//!   node per entity carrying the entity key, `state`, `previousState`,
//!   `enteredAt`, and the pass-through fields from the triggering query row.
//!   Late-joining subscribers are bootstrapped from the persisted state.
//!
//! Because the component graph requires globally-unique component ids, the
//! reaction and the source use **different** ids; the reaction's `source_id`
//! config names the source. They rendezvous in-process via [`registry`].
//!
//! ## Example
//!
//! ```no_run
//! use drasi_reaction_state_machine::{StateMachineBuilder, config::{StateDef, EnterCondition, Op}};
//!
//! # fn build() -> anyhow::Result<()> {
//! let (reaction, source) = StateMachineBuilder::new("order-state")
//!     .with_source_id("order-state-source")
//!     .with_entity_label("OrderState")
//!     .with_key_field("orderId")
//!     .with_state(StateDef {
//!         id: "NEW".to_string(),
//!         enter: vec![EnterCondition {
//!             query: "draft-orders".to_string(),
//!             previous: vec![],
//!             key: "{{orderId}}".to_string(),
//!             ops: vec![Op::Added],
//!         }],
//!     })
//!     .build_pair()?;
//! // drasi.add_reaction(reaction).await?; drasi.add_source(source).await?;
//! # let _ = (reaction, source);
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod descriptor;
pub mod engine;
pub mod reaction;
pub mod registry;
pub mod source;

#[cfg(test)]
mod tests;

pub use config::{EnterCondition, Op, StateDef, StateMachineReactionConfig};
pub use engine::{EntityRecord, StateMachine};
pub use reaction::StateMachineReaction;
pub use source::StateMachineSource;

/// Builder that produces a linked [`StateMachineReaction`] and
/// [`StateMachineSource`] pair.
pub struct StateMachineBuilder {
    reaction_id: String,
    config: StateMachineReactionConfig,
    priority_queue_capacity: Option<usize>,
    auto_start: bool,
}

impl StateMachineBuilder {
    /// Start building a state machine with the given reaction id.
    pub fn new(reaction_id: impl Into<String>) -> Self {
        Self {
            reaction_id: reaction_id.into(),
            config: StateMachineReactionConfig::default(),
            priority_queue_capacity: None,
            auto_start: true,
        }
    }

    /// Set the id under which the companion entity-state source is registered.
    pub fn with_source_id(mut self, source_id: impl Into<String>) -> Self {
        self.config.source_id = source_id.into();
        self
    }

    /// Set the label applied to emitted entity-state nodes.
    pub fn with_entity_label(mut self, label: impl Into<String>) -> Self {
        self.config.entity_label = label.into();
        self
    }

    /// Set the node property name that holds the entity key.
    pub fn with_key_field(mut self, key_field: impl Into<String>) -> Self {
        self.config.key_field = key_field.into();
        self
    }

    /// Add a state definition.
    pub fn with_state(mut self, state: StateDef) -> Self {
        self.config.states.push(state);
        self
    }

    /// Replace the entire configuration.
    pub fn with_config(mut self, config: StateMachineReactionConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the reaction's priority queue capacity.
    pub fn with_priority_queue_capacity(mut self, capacity: usize) -> Self {
        self.priority_queue_capacity = Some(capacity);
        self
    }

    /// Set whether the reaction and source auto-start.
    pub fn with_auto_start(mut self, auto_start: bool) -> Self {
        self.auto_start = auto_start;
        self
    }

    /// Build the linked reaction and source.
    ///
    /// Both must be added to the same `DrasiLib` instance (via `add_reaction` and
    /// `add_source`). The source must use the configured `source_id`.
    pub fn build_pair(self) -> anyhow::Result<(StateMachineReaction, StateMachineSource)> {
        let reaction = StateMachineReaction::create(
            self.reaction_id,
            self.config.clone(),
            self.priority_queue_capacity,
            self.auto_start,
        )?;
        let source = StateMachineSource::create(&self.config.source_id, self.auto_start)?;
        Ok((reaction, source))
    }
}

/// Dynamic plugin entry point. Exports both the reaction and source descriptors
/// (both of kind `state-machine`) from a single plugin library.
#[cfg(feature = "dynamic-plugin")]
drasi_plugin_sdk::export_plugin!(
    plugin_id = "state-machine-reaction",
    core_version = env!("CARGO_PKG_VERSION"),
    lib_version = env!("CARGO_PKG_VERSION"),
    plugin_version = env!("CARGO_PKG_VERSION"),
    source_descriptors = [descriptor::StateMachineSourceDescriptor],
    reaction_descriptors = [descriptor::StateMachineReactionDescriptor],
    bootstrap_descriptors = [],
);

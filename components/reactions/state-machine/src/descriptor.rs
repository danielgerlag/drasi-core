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

//! Plugin descriptors for the state machine reaction and its companion source.
//!
//! A single plugin crate exports **both** a reaction descriptor and a source
//! descriptor (each with kind `state-machine`). The reaction config carries a
//! `sourceId`; the user declares a source of kind `state-machine` with that id so
//! downstream queries can subscribe to it. The two link at runtime via the
//! in-process [`crate::registry`].

use drasi_lib::{Reaction, Source};
use drasi_plugin_sdk::prelude::*;
use utoipa::OpenApi;

use crate::config::{EnterCondition, Op, StateDef, StateMachineReactionConfig};
use crate::{StateMachineReaction, StateMachineSource};

#[derive(OpenApi)]
#[openapi(components(schemas(StateMachineReactionConfig, StateDef, EnterCondition, Op)))]
struct StateMachineReactionSchemas;

/// Descriptor for the state machine reaction plugin.
pub struct StateMachineReactionDescriptor;

#[async_trait]
impl ReactionPluginDescriptor for StateMachineReactionDescriptor {
    fn kind(&self) -> &str {
        "state-machine"
    }

    fn config_version(&self) -> &str {
        "1.0.0"
    }

    fn config_schema_name(&self) -> &str {
        "reaction.state_machine.StateMachineReactionConfig"
    }

    fn config_schema_json(&self) -> String {
        let api = StateMachineReactionSchemas::openapi();
        serde_json::to_string(
            &api.components
                .as_ref()
                .expect("OpenAPI components missing")
                .schemas,
        )
        .expect("Failed to serialize config schema")
    }

    fn display_name(&self) -> &str {
        "State Machine"
    }

    fn display_description(&self) -> &str {
        "Maps query results to entity state transitions and exposes live entity state as a source"
    }

    async fn create_reaction(
        &self,
        id: &str,
        _query_ids: Vec<String>,
        config_json: &serde_json::Value,
        auto_start: bool,
    ) -> anyhow::Result<Box<dyn Reaction>> {
        // The subscribed queries are derived from the `states` config, so the
        // host-supplied `query_ids` (from the YAML `queries:` list) are advisory.
        let config: StateMachineReactionConfig = serde_json::from_value(config_json.clone())?;
        let reaction = StateMachineReaction::create(id.to_string(), config, None, auto_start)?;
        Ok(Box::new(reaction))
    }
}

/// Configuration DTO for the state machine source.
///
/// The source is self-describing (persisted entity records carry their own label
/// and key field), so it requires no configuration. This empty DTO exists only to
/// publish a schema for the `state-machine` source kind.
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(as = source::state_machine::StateMachineSourceConfig)]
#[serde(rename_all = "camelCase", default)]
pub struct StateMachineSourceConfigDto {}

#[derive(OpenApi)]
#[openapi(components(schemas(StateMachineSourceConfigDto)))]
struct StateMachineSourceSchemas;

/// Descriptor for the state machine source plugin.
pub struct StateMachineSourceDescriptor;

#[async_trait]
impl SourcePluginDescriptor for StateMachineSourceDescriptor {
    fn kind(&self) -> &str {
        "state-machine"
    }

    fn config_version(&self) -> &str {
        "1.0.0"
    }

    fn config_schema_name(&self) -> &str {
        "source.state_machine.StateMachineSourceConfig"
    }

    fn config_schema_json(&self) -> String {
        let api = StateMachineSourceSchemas::openapi();
        serde_json::to_string(
            &api.components
                .as_ref()
                .expect("OpenAPI components missing")
                .schemas,
        )
        .expect("Failed to serialize config schema")
    }

    fn display_name(&self) -> &str {
        "State Machine"
    }

    fn display_description(&self) -> &str {
        "Exposes live entity state produced by a state machine reaction"
    }

    async fn create_source(
        &self,
        id: &str,
        _config_json: &serde_json::Value,
        auto_start: bool,
    ) -> anyhow::Result<Box<dyn Source>> {
        let source = StateMachineSource::create(id, auto_start)?;
        Ok(Box::new(source))
    }
}

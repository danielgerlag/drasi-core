// Copyright 2026 The Drasi Authors.
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

//! Descriptor for the dashboard reaction plugin.

use drasi_lib::reactions::Reaction;
use drasi_plugin_sdk::prelude::*;
use utoipa::OpenApi;

use crate::DashboardReactionBuilder;

/// Dashboard reaction config DTO.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(as = reaction::dashboard::DashboardReactionConfig)]
#[serde(rename_all = "camelCase")]
pub struct DashboardReactionConfigDto {
    /// Host to bind dashboard server.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<ConfigValueString>)]
    pub host: Option<ConfigValue<String>>,

    /// Port to bind dashboard server.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<ConfigValueU16>)]
    pub port: Option<ConfigValue<u16>>,

    /// WebSocket heartbeat interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<ConfigValueU64>)]
    pub heartbeat_interval_ms: Option<ConfigValue<u64>>,
}

#[derive(OpenApi)]
#[openapi(components(schemas(DashboardReactionConfigDto,)))]
struct DashboardReactionSchemas;

/// Descriptor for creating dashboard reaction instances.
pub struct DashboardReactionDescriptor;

#[async_trait]
impl ReactionPluginDescriptor for DashboardReactionDescriptor {
    fn kind(&self) -> &str {
        "dashboard"
    }

    fn config_version(&self) -> &str {
        "1.0.0"
    }

    fn config_schema_name(&self) -> &str {
        "reaction.dashboard.DashboardReactionConfig"
    }

    fn config_schema_json(&self) -> String {
        let api = DashboardReactionSchemas::openapi();
        serde_json::to_string(
            &api.components
                .as_ref()
                .expect("OpenAPI components missing")
                .schemas,
        )
        .expect("Failed to serialize config schema")
    }

    async fn create_reaction(
        &self,
        id: &str,
        query_ids: Vec<String>,
        config_json: &serde_json::Value,
        auto_start: bool,
    ) -> anyhow::Result<Box<dyn Reaction>> {
        let dto: DashboardReactionConfigDto = serde_json::from_value(config_json.clone())?;
        let mapper = DtoMapper::new();

        let mut builder = DashboardReactionBuilder::new(id)
            .with_queries(query_ids)
            .with_auto_start(auto_start);

        if let Some(host) = &dto.host {
            builder = builder.with_host(mapper.resolve_string(host)?);
        }
        if let Some(port) = &dto.port {
            builder = builder.with_port(mapper.resolve_typed(port)?);
        }
        if let Some(heartbeat_interval_ms) = &dto.heartbeat_interval_ms {
            builder =
                builder.with_heartbeat_interval_ms(mapper.resolve_typed(heartbeat_interval_ms)?);
        }

        Ok(Box::new(builder.build()?))
    }
}

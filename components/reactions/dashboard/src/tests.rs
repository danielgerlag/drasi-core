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

use super::*;
use drasi_lib::Reaction;

#[test]
fn test_dashboard_builder_defaults() {
    let reaction = DashboardReactionBuilder::new("dashboard-test")
        .build()
        .expect("builder should succeed");
    assert_eq!(reaction.id(), "dashboard-test");
    assert_eq!(reaction.type_name(), "dashboard");

    let properties = reaction.properties();
    assert_eq!(
        properties.get("host"),
        Some(&serde_json::Value::String("0.0.0.0".to_string()))
    );
    assert_eq!(
        properties.get("port"),
        Some(&serde_json::Value::Number(3000_u16.into()))
    );
    assert_eq!(
        properties.get("heartbeat_interval_ms"),
        Some(&serde_json::Value::Number(30_000_u64.into()))
    );
}

#[test]
fn test_dashboard_builder_custom_values() {
    let reaction = DashboardReaction::builder("dashboard-custom")
        .with_query("q1")
        .with_host("127.0.0.1")
        .with_port(18000)
        .with_heartbeat_interval_ms(5000)
        .with_priority_queue_capacity(500)
        .with_auto_start(false)
        .build()
        .expect("builder should succeed");

    assert_eq!(reaction.id(), "dashboard-custom");
    assert_eq!(reaction.query_ids(), vec!["q1".to_string()]);
    assert!(!reaction.auto_start());

    let properties = reaction.properties();
    assert_eq!(
        properties.get("host"),
        Some(&serde_json::Value::String("127.0.0.1".to_string()))
    );
    assert_eq!(
        properties.get("port"),
        Some(&serde_json::Value::Number(18_000_u16.into()))
    );
}

#[test]
fn test_dashboard_config_serialization() {
    let config = DashboardReactionConfig {
        host: "localhost".to_string(),
        port: 5050,
        heartbeat_interval_ms: 15_000,
        results_api_url: None,
    };

    let serialized = serde_json::to_string(&config).expect("config should serialize");
    let deserialized: DashboardReactionConfig =
        serde_json::from_str(&serialized).expect("config should deserialize");
    assert_eq!(config, deserialized);
}

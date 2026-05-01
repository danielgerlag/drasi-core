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

//! # DrasiLib Dashboard Example
//!
//! This example demonstrates a real-time stock dashboard built with drasi-lib.
//! It shows how to wire an HTTP source through multiple continuous Cypher queries
//! into a Dashboard reaction that serves a visual web UI.
//!
//! ## Running
//!
//! ```bash
//! cargo run
//! ```
//!
//! Then open http://localhost:3000 in your browser to access the dashboard.
//!
//! ## Architecture
//!
//! HTTP Source (port 9000) → 4 Cypher Queries → Dashboard Reaction (port 3000)
//!
//! ## Testing
//!
//! Use the change.http file with VS Code REST Client or curl to send stock
//! price updates and watch them appear on the dashboard in real time.

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use drasi_lib::{DrasiLib, Query};
use drasi_reaction_dashboard::{DashboardReaction, DashboardConfig, DashboardWidget, GridOptions, WidgetGrid};
use drasi_source_http::HttpSource;
use drasi_bootstrap_scriptfile::ScriptFileBootstrapProvider;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    println!("╔════════════════════════════════════════════╗");
    println!("║     DrasiLib Dashboard Example             ║");
    println!("╚════════════════════════════════════════════╝\n");

    // =========================================================================
    // Step 1: Create Bootstrap Provider
    // =========================================================================
    let bootstrap_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bootstrap_data.jsonl");

    println!("Loading bootstrap data from: {}", bootstrap_path.display());

    let bootstrap_provider = ScriptFileBootstrapProvider::builder()
        .with_file(bootstrap_path.to_string_lossy().to_string())
        .build();

    // =========================================================================
    // Step 2: Create HTTP Source
    // =========================================================================
    // Accepts stock price events on port 9000.
    // POST /sources/stock-prices/events for single events
    // POST /sources/stock-prices/events/batch for batch updates

    let http_source = HttpSource::builder("stock-prices")
        .with_host("0.0.0.0")
        .with_port(9000)
        .with_adaptive_enabled(true)
        .with_adaptive_max_batch_size(100)
        .with_adaptive_min_batch_size(1)
        .with_adaptive_max_wait_ms(50)
        .with_adaptive_min_wait_ms(10)
        .with_bootstrap_provider(bootstrap_provider)
        .build()?;

    // =========================================================================
    // Step 3: Define Queries
    // =========================================================================
    // Four queries that each provide a different perspective on the stock data,
    // ideal for different dashboard widget types.

    // Query 1: All Prices — full stock table (good for Table widget)
    let all_prices_query = Query::cypher("all-prices")
        .query(r#"
            MATCH (sp:stock_prices)
            RETURN sp.symbol AS symbol,
                   sp.price AS price,
                   sp.previous_close AS previous_close,
                   sp.volume AS volume
        "#)
        .from_source("stock-prices")
        .auto_start(true)
        .enable_bootstrap(true)
        .build();

    // Query 2: Gainers — stocks beating previous close (good for Bar Chart)
    let gainers_query = Query::cypher("gainers")
        .query(r#"
            MATCH (sp:stock_prices)
            WHERE sp.price > sp.previous_close
            RETURN sp.symbol AS symbol,
                   sp.price AS price,
                   sp.previous_close AS previous_close,
                   ((sp.price - sp.previous_close) / sp.previous_close * 100) AS gain_percent
        "#)
        .from_source("stock-prices")
        .auto_start(true)
        .enable_bootstrap(true)
        .build();

    // Query 3: High Volume — stocks with volume > 1M (good for Table + KPI)
    let high_volume_query = Query::cypher("high-volume")
        .query(r#"
            MATCH (sp:stock_prices)
            WHERE sp.volume > 1000000
            RETURN sp.symbol AS symbol,
                   sp.price AS price,
                   sp.volume AS volume
        "#)
        .from_source("stock-prices")
        .auto_start(true)
        .enable_bootstrap(true)
        .build();

    // Query 4: Top Movers — biggest price changes as % (good for Gauge/KPI)
    let top_movers_query = Query::cypher("top-movers")
        .query(r#"
            MATCH (sp:stock_prices)
            WHERE sp.previous_close > 0
            RETURN sp.symbol AS symbol,
                   sp.price AS price,
                   sp.previous_close AS previous_close,
                   abs((sp.price - sp.previous_close) / sp.previous_close * 100) AS change_percent
        "#)
        .from_source("stock-prices")
        .auto_start(true)
        .enable_bootstrap(true)
        .build();

    // =========================================================================
    // Step 4: Create Dashboard Reaction
    // =========================================================================
    // The Dashboard reaction serves a web UI on port 3000 with:
    //   - Visual drag-and-drop dashboard designer
    //   - Real-time data via WebSocket
    //   - 8 widget types (charts, tables, gauges, KPIs, etc.)
    //   - Predefined dashboards seeded from config

    let predefined_dashboard = DashboardConfig::with_id(
        "stock-overview",
        "Stock Market Overview".to_string(),
        GridOptions::default(),
        vec![
            DashboardWidget {
                id: "w-table-all".to_string(),
                widget_type: "table".to_string(),
                title: "All Stocks".to_string(),
                grid: WidgetGrid { x: 0, y: 0, w: 8, h: 4 },
                config: serde_json::json!({
                    "queryId": "all-prices",
                    "columns": ["symbol", "price", "previous_close", "volume"]
                }),
            },
            DashboardWidget {
                id: "w-kpi-count".to_string(),
                widget_type: "kpi".to_string(),
                title: "Total Stocks".to_string(),
                grid: WidgetGrid { x: 8, y: 0, w: 4, h: 2 },
                config: serde_json::json!({
                    "queryId": "all-prices",
                    "valueField": "symbol",
                    "aggregation": "count",
                    "label": "Total Stocks"
                }),
            },
            DashboardWidget {
                id: "w-kpi-highvol".to_string(),
                widget_type: "kpi".to_string(),
                title: "High Volume Stocks".to_string(),
                grid: WidgetGrid { x: 8, y: 2, w: 4, h: 2 },
                config: serde_json::json!({
                    "queryId": "high-volume",
                    "valueField": "symbol",
                    "aggregation": "count",
                    "label": "High Volume"
                }),
            },
            DashboardWidget {
                id: "w-bar-gainers".to_string(),
                widget_type: "bar_chart".to_string(),
                title: "Gainers (%)".to_string(),
                grid: WidgetGrid { x: 0, y: 4, w: 6, h: 4 },
                config: serde_json::json!({
                    "queryId": "gainers",
                    "categoryField": "symbol",
                    "valueFields": ["gain_percent"]
                }),
            },
            DashboardWidget {
                id: "w-gauge-top".to_string(),
                widget_type: "gauge".to_string(),
                title: "Max Change %".to_string(),
                grid: WidgetGrid { x: 6, y: 4, w: 3, h: 4 },
                config: serde_json::json!({
                    "queryId": "top-movers",
                    "valueField": "change_percent",
                    "aggregation": "max",
                    "min": 0,
                    "max": 20
                }),
            },
            DashboardWidget {
                id: "w-md-summary".to_string(),
                widget_type: "text".to_string(),
                title: "Market Summary".to_string(),
                grid: WidgetGrid { x: 9, y: 4, w: 3, h: 4 },
                config: serde_json::json!({
                    "queryId": "all-prices",
                    "template": "## Market Snapshot\n\n**{{count}}** stocks tracked\n\n| Metric | Value |\n|--------|-------|\n| Avg Price | {{format (avg \"price\") \"currency\"}} |\n| Total Volume | {{format (sum \"volume\") \"compact\"}} |\n"
                }),
            },
        ],
    );

    let dashboard_reaction = DashboardReaction::builder("stock-dashboard")
        .with_query("all-prices")
        .with_query("gainers")
        .with_query("high-volume")
        .with_query("top-movers")
        .with_host("0.0.0.0")
        .with_port(3000)
        .with_results_api_url("http://localhost:8080")
        .with_dashboard(predefined_dashboard)
        .build()?;

    // =========================================================================
    // Step 5: Build DrasiLib
    // =========================================================================
    let core = Arc::new(
        DrasiLib::builder()
            .with_id("stock-dashboard-app")
            .with_source(http_source)
            .with_query(all_prices_query)
            .with_query(gainers_query)
            .with_query(high_volume_query)
            .with_query(top_movers_query)
            .with_reaction(dashboard_reaction)
            .build()
            .await?
    );

    // =========================================================================
    // Step 6: Start Processing
    // =========================================================================
    core.start().await?;

    // =========================================================================
    // Step 7: Start Results API Server (optional debugging)
    // =========================================================================
    let api_core = core.clone();
    let results_api = Router::new()
        .route("/queries/:id/results", get(get_query_results))
        .with_state(api_core);

    let api_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
        axum::serve(listener, results_api).await.unwrap();
    });

    println!("\n┌──────────────────────────────────────────────┐");
    println!("│ Stock Dashboard Started!                      │");
    println!("├──────────────────────────────────────────────┤");
    println!("│ 🌐 Dashboard UI: http://localhost:3000       │");
    println!("├──────────────────────────────────────────────┤");
    println!("│ HTTP Source: http://localhost:9000            │");
    println!("│   POST /sources/stock-prices/events           │");
    println!("│   POST /sources/stock-prices/events/batch     │");
    println!("├──────────────────────────────────────────────┤");
    println!("│ Queries:                                      │");
    println!("│   • all-prices  — All stocks (Table)          │");
    println!("│   • gainers     — Stocks with gains (Bar)     │");
    println!("│   • high-volume — Volume > 1M (KPI)           │");
    println!("│   • top-movers  — Price changes % (Gauge)     │");
    println!("├──────────────────────────────────────────────┤");
    println!("│ Results API: http://localhost:8080            │");
    println!("│   GET /queries/<id>/results                   │");
    println!("├──────────────────────────────────────────────┤");
    println!("│ Press Ctrl+C to stop                          │");
    println!("└──────────────────────────────────────────────┘\n");

    tokio::signal::ctrl_c().await?;

    println!("\n>>> Shutting down gracefully...");
    api_handle.abort();
    core.stop().await?;
    println!(">>> Shutdown complete.");

    Ok(())
}

/// Handler for GET /queries/:id/results
async fn get_query_results(
    State(core): State<Arc<DrasiLib>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, (axum::http::StatusCode, String)> {
    core.get_query_results(&id)
        .await
        .map(Json)
        .map_err(|e| (axum::http::StatusCode::NOT_FOUND, e.to_string()))
}

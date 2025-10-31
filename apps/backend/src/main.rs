use anyhow::Context;
use axum::Router;
use backend::api::rest;
use backend::api::ws;
use backend::config::Config;
use backend::db::Db;
use backend::engine::MatchingEngine;
use backend::models::domain::{EngineEvent, EngineRequest};
use backend::AppState;
use tokio::sync::{broadcast, mpsc};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables (for secrets/overrides)
    let _ = dotenvy::from_path(".env.defaults");
    let _ = dotenvy::from_path_override(".env");

    env_logger::init();

    // ===============================
    // Load configuration
    // ===============================
    let config = Config::load().context("Failed to load configuration")?;
    let host = std::env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let addr = format!("{}:{}", host, port);

    log::info!("Starting exchange with configuration:");
    log::info!("  Markets: {}", config.markets.len());
    log::info!("  Tokens: {}", config.tokens.len());

    // ===============================
    // Connect to databases
    // ===============================
    let db = Db::connect()
        .await
        .context("Failed to connect to databases")?;
    log::info!("Connected to PostgreSQL and ClickHouse");

    // ===============================
    // Create engine channels
    // ===============================
    let (engine_tx, engine_rx) = mpsc::channel::<EngineRequest>(100);
    let (event_tx, _) = broadcast::channel::<EngineEvent>(1000); // use event_tx to create more listeners

    // ===============================
    // Run matching engine
    // ===============================
    let engine = MatchingEngine::new(db.clone(), engine_rx, event_tx.clone());

    tokio::spawn(async move {
        engine.run().await;
    });

    // ===============================
    // Create axum app
    // ===============================
    let rest = rest::create_rest();
    let ws = ws::create_ws();
    let state = AppState {
        db,
        engine_tx,
        event_tx,
    };

    let app = Router::new()
        .merge(rest)
        .merge(ws)
        .with_state(state)
        .layer(CorsLayer::permissive());

    // ===============================
    // Start server
    // ===============================
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind to {}", addr))?;

    println!("\n🚀 Backend server running on http://{}", addr);
    println!("📖 OpenAPI docs: http://{}/api/docs", addr);
    println!("📋 OpenAPI spec: http://{}/api/openapi.json", addr);
    println!("\n💡 Tip: Run 'just db-init' to initialize markets and tokens\n");

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}

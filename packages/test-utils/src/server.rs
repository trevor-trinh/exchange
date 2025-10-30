use crate::containers::TestContainers;
use axum::Router;
use backend::api::{rest, ws};
use backend::engine::MatchingEngine;
use backend::AppState;
use tokio::sync::{broadcast, mpsc};
use tower_http::cors::CorsLayer;

/// Handle to a running test server
pub struct TestServer {
    pub base_url: String,
    pub ws_url: String,
    _containers: TestContainers,
    _shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl TestServer {
    /// Start a test HTTP server on a random available port
    ///
    /// Mimics the production server setup main.rs
    ///
    /// This spawns:
    /// - PostgreSQL and ClickHouse containers
    /// - Matching engine
    /// - Axum HTTP server with REST + WebSocket routes
    ///
    /// The server runs in the background and will shutdown when dropped.
    pub async fn start() -> anyhow::Result<Self> {
        // Setup database containers
        let containers = TestContainers::setup().await?;

        // Setup matching engine
        let (engine_tx, engine_rx) = mpsc::channel(100);
        let (event_tx, _event_rx) = broadcast::channel(1000);

        let engine = MatchingEngine::new(containers.db_clone(), engine_rx, event_tx.clone());
        tokio::spawn(async move {
            engine.run().await;
        });

        // Create REST and WebSocket routes
        let rest_routes = rest::create_rest();
        let ws_routes = ws::create_ws();
        let state = AppState {
            db: containers.db_clone(),
            engine_tx,
            event_tx,
        };
        let app = Router::new()
            .merge(rest_routes)
            .merge(ws_routes)
            .with_state(state)
            .layer(CorsLayer::permissive());

        // Bind to random available port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind test server: {}", e))?;

        let addr = listener
            .local_addr()
            .map_err(|e| anyhow::anyhow!("Failed to get local address: {}", e))?;

        let base_url = format!("http://{}", addr);
        let ws_url = format!("ws://{}/ws", addr);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn server in background
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .expect("Server failed to start");
        });

        // Give server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(Self {
            base_url,
            ws_url,
            _containers: containers,
            _shutdown_tx: shutdown_tx,
        })
    }

    /// Build full HTTP URL for a path
    ///
    /// # Example
    /// ```
    /// let url = server.url("/api/health");
    /// let response = reqwest::get(&url).await?;
    /// ```
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

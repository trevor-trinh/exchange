use backend::db::Db;
use backend::models::domain::{EngineEvent, EngineRequest, Order, OrderStatus, OrderType, Side};
use chrono::Utc;
use exchange_test_utils::TestContainers;
use tokio::sync::{broadcast, mpsc, oneshot};
use uuid::Uuid;

use axum::Router;
use backend::api::rest;
use backend::api::ws;
use backend::engine::MatchingEngine;
use backend::AppState;
use tower_http::cors::CorsLayer;

#[allow(dead_code)]
pub struct TestServer {
    pub address: String,
    pub test_db: TestDb,
    pub test_engine: TestEngine,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

#[allow(dead_code)]
impl TestServer {
    /// Start a test HTTP server on a random available port
    /// Uses TestEngine internally for matching engine setup
    pub async fn start() -> anyhow::Result<Self> {
        // Setup database
        let test_db = TestDb::setup().await?;

        // Setup matching engine using TestEngine (without creating users)
        // Integration tests will create their own users
        let test_engine = TestEngine::new_with_users(&test_db, false).await;

        // Create REST and WebSocket routes
        let rest = rest::create_rest();
        let ws = ws::create_ws();
        let state = AppState {
            db: test_engine.db.clone(),
            engine_tx: test_engine.engine_tx.clone(),
            event_tx: test_engine.event_tx(),
        };
        let app = Router::new()
            .merge(rest)
            .merge(ws)
            .with_state(state)
            .layer(CorsLayer::permissive());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind test server: {}", e))?;

        let addr = listener
            .local_addr()
            .map_err(|e| anyhow::anyhow!("Failed to get local address: {}", e))?;

        let address = format!("http://{}", addr);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn server in background
        // in main.rs we spawn on main thread instead
        // here is because we need main thread to run tests
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
            address,
            test_db,
            test_engine,
            shutdown_tx,
        })
    }

    // ============================================================================
    // URL Builders - Use these with raw reqwest/tokio-tungstenite in tests
    // ============================================================================

    /// Build full HTTP URL for a path
    ///
    /// # Example
    /// ```
    /// let url = server.url("/api/health");
    /// let response = reqwest::get(&url).await?;
    /// ```
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }

    /// Build WebSocket URL for a path
    ///
    /// # Example
    /// ```
    /// let ws_url = server.ws_url("/ws");
    /// let (ws, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    /// ```
    pub fn ws_url(&self, path: &str) -> String {
        format!("{}{}", self.address.replace("http://", "ws://"), path)
    }

    // ============================================================================
    // Database Access - Backend tests can access internals
    // ============================================================================

    /// Get reference to database connection for direct DB operations in tests
    pub fn db(&self) -> &Db {
        &self.test_db.db
    }

    /// Get reference to test engine for engine operations in tests
    pub fn engine(&self) -> &TestEngine {
        &self.test_engine
    }
}

/// Test database container setup
///
/// This wraps the shared TestContainers but exposes the internal Db
/// for backend tests that need to verify internal state.
#[allow(dead_code)]
pub struct TestDb {
    pub db: Db,
    _containers: TestContainers,
}

#[allow(dead_code)]
impl TestDb {
    /// Set up test databases with containers
    pub async fn setup() -> anyhow::Result<Self> {
        let containers = TestContainers::setup().await?;
        let db = containers.db_clone();

        Ok(TestDb {
            db,
            _containers: containers,
        })
    }

    /// Helper function to create test user
    pub async fn create_test_user(
        self: &TestDb,
        address: &str,
    ) -> anyhow::Result<backend::models::domain::User> {
        self.db
            .create_user(address.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create test user: {}", e))
    }

    /// Helper function to create test token
    pub async fn create_test_token(
        self: &TestDb,
        ticker: &str,
        decimals: u8,
        name: &str,
    ) -> anyhow::Result<backend::models::domain::Token> {
        self.db
            .create_token(ticker.to_string(), decimals, name.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create test token: {}", e))
    }

    /// Helper function to create test market
    /// Requires that both base and quote tokens already exist
    pub async fn create_test_market(
        self: &TestDb,
        base_ticker: &str,
        quote_ticker: &str,
    ) -> anyhow::Result<backend::models::domain::Market> {
        self.db
            .create_market(
                base_ticker.to_string(),
                quote_ticker.to_string(),
                1000,    // tick_size
                1000000, // lot_size
                1000000, // min_size
                10,      // maker_fee_bps
                20,      // taker_fee_bps
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create test market: {}", e))
    }

    /// Helper function to create test market with tokens
    /// Creates the tokens first, then the market
    pub async fn create_test_market_with_tokens(
        self: &TestDb,
        base_ticker: &str,
        quote_ticker: &str,
    ) -> anyhow::Result<backend::models::domain::Market> {
        // Create tokens with realistic decimals
        // Base tokens (crypto assets) typically use 8 decimals
        // Quote tokens (stablecoins) typically use 6 decimals
        // This prevents integer truncation issues in price calculations
        let base_decimals = 8;
        let quote_decimals = 6;

        self.create_test_token(
            base_ticker,
            base_decimals,
            &format!("{} Token", base_ticker),
        )
        .await?;
        self.create_test_token(
            quote_ticker,
            quote_decimals,
            &format!("{} Token", quote_ticker),
        )
        .await?;

        // Then create the market
        self.create_test_market(base_ticker, quote_ticker).await
    }

    /// Helper function to create test candle
    pub async fn create_test_candle(
        self: &TestDb,
        market_id: &str,
        timestamp: chrono::DateTime<chrono::Utc>,
        interval: &str,                        // '1m', '5m', '15m', '1h', '1d'
        ohlcv: (u128, u128, u128, u128, u128), // (open, high, low, close, volume)
    ) -> anyhow::Result<()> {
        self.db
            .insert_candle(
                market_id.to_string(),
                timestamp,
                interval.to_string(),
                ohlcv,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create test candle: {}", e))
    }
}

/// Helper for matching engine testing
///
/// This exposes internal engine channels and domain types
/// for backend tests that need to test matching logic directly.
#[allow(dead_code)]
pub struct TestEngine {
    pub db: Db,
    pub engine_tx: mpsc::Sender<EngineRequest>,
    pub event_rx: broadcast::Receiver<EngineEvent>,
    event_tx: broadcast::Sender<EngineEvent>,
}

#[allow(dead_code)]
impl TestEngine {
    pub async fn new(test_db: &TestDb) -> Self {
        Self::new_with_users(test_db, true).await
    }

    /// Create a new TestEngine, optionally creating common test users
    pub async fn new_with_users(test_db: &TestDb, create_users: bool) -> Self {
        // Create common test users for engine tests only
        if create_users {
            let users = vec![
                "buyer",
                "seller",
                "seller1",
                "seller2",
                "seller3",
                "buyer1",
                "buyer2",
                "buyer3",
                "alice",
                "bob",
                "charlie",
                "dave",
                "user1",
                "user2",
                "attacker",
                "big_buyer",
            ];

            for user in &users {
                let _ = test_db.create_test_user(user).await;
            }

            // Give each user generous balances for various common test tokens
            // Using realistic decimals: 8 for base tokens, 6 for quote tokens
            let test_tokens = vec![
                ("BTC", 8, 1_000_000_000u128),     // 10 BTC
                ("ETH", 8, 10_000_000_000),        // 100 ETH
                ("SOL", 8, 100_000_000_000),       // 1,000 SOL
                ("LINK", 8, 1_000_000_000_000),    // 10,000 LINK
                ("ADA", 8, 10_000_000_000_000),    // 100,000 ADA
                ("MATIC", 8, 100_000_000_000_000), // 1,000,000 MATIC
                ("ATOM", 8, 10_000_000_000_000),   // 100,000 ATOM
                ("AVAX", 8, 10_000_000_000_000),   // 100,000 AVAX
                ("DOT", 8, 10_000_000_000_000),    // 100,000 DOT
                ("UNI", 8, 10_000_000_000_000),    // 100,000 UNI
                ("USDC", 6, 10_000_000_000_000),   // 10,000,000 USDC
                ("USDT", 6, 10_000_000_000_000),   // 10,000,000 USDT
                ("DAI", 6, 10_000_000_000_000),    // 10,000,000 DAI
            ];

            for user in &users {
                for (ticker, _decimals, amount) in &test_tokens {
                    // Create token if it doesn't exist (ignore errors if it already exists)
                    if test_db.db.get_token(ticker).await.is_err() {
                        let _ = test_db
                            .db
                            .create_token(
                                ticker.to_string(),
                                *_decimals,
                                format!("{} Token", ticker),
                            )
                            .await;
                    }

                    // Add balance to user
                    let _ = test_db.db.add_balance(user, ticker, *amount).await;
                }
            }
        }

        let (engine_tx, engine_rx) = mpsc::channel::<EngineRequest>(100);
        let (event_tx, event_rx) = broadcast::channel::<EngineEvent>(1000);

        let engine = MatchingEngine::new(test_db.db.clone(), engine_rx, event_tx.clone());

        // Spawn engine in background
        tokio::spawn(async move {
            engine.run().await;
        });

        Self {
            db: test_db.db.clone(),
            engine_tx,
            event_rx,
            event_tx,
        }
    }

    /// Get a clone of the event sender for HTTP server state
    pub fn event_tx(&self) -> broadcast::Sender<EngineEvent> {
        self.event_tx.clone()
    }

    /// Helper to place an order and get the response
    pub async fn place_order(
        &self,
        order: Order,
    ) -> Result<backend::models::api::OrderPlaced, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.engine_tx
            .send(EngineRequest::PlaceOrder { order, response_tx })
            .await
            .map_err(|e| format!("Failed to send order: {}", e))?;

        response_rx
            .await
            .map_err(|e| format!("Failed to receive response: {}", e))?
            .map_err(|e| format!("Order placement failed: {}", e))
    }

    /// Helper to cancel an order
    pub async fn cancel_order(
        &self,
        order_id: Uuid,
        user_address: String,
    ) -> Result<backend::models::api::OrderCancelled, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.engine_tx
            .send(EngineRequest::CancelOrder {
                order_id,
                user_address,
                response_tx,
            })
            .await
            .map_err(|e| format!("Failed to send cancel request: {}", e))?;

        response_rx
            .await
            .map_err(|e| format!("Failed to receive response: {}", e))?
            .map_err(|e| format!("Order cancellation failed: {}", e))
    }

    /// Helper to create a test order
    pub fn create_order(
        user_address: &str,
        market_id: &str,
        side: Side,
        order_type: OrderType,
        price: u128,
        size: u128,
    ) -> Order {
        Order {
            id: Uuid::new_v4(),
            user_address: user_address.to_string(),
            market_id: market_id.to_string(),
            price,
            size,
            side,
            order_type,
            status: OrderStatus::Pending,
            filled_size: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

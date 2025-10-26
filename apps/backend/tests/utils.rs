use backend::db::Db;
use clickhouse::Client;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::{clickhouse::ClickHouse, postgres::Postgres};

use axum::Router;
use backend::api::rest;
use backend::api::ws;
use backend::engine::MatchingEngine;
use backend::models::domain::{EngineEvent, EngineRequest};
use backend::AppState;
use tokio::sync::{broadcast, mpsc};
use tower_http::cors::CorsLayer;

/// Test database container setup
#[allow(dead_code)]
pub struct TestDb {
    pub db: Db,
    postgres_container: testcontainers::ContainerAsync<Postgres>,
    clickhouse_container: testcontainers::ContainerAsync<ClickHouse>,
}

#[allow(dead_code)]
impl TestDb {
    /// Set up test databases with containers
    pub async fn setup() -> anyhow::Result<Self> {
        // ================================ Start containers ================================
        let postgres_container = Postgres::default()
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start PostgreSQL container: {}", e))?;

        let clickhouse_container = ClickHouse::default()
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start ClickHouse container: {}", e))?;

        let postgres_port = postgres_container
            .get_host_port_ipv4(5432)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get PostgreSQL port: {}", e))?;

        let clickhouse_port = clickhouse_container
            .get_host_port_ipv4(8123)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get ClickHouse port: {}", e))?;

        // ================================ Create database connections ================================
        let postgres_url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            postgres_container.get_host().await.unwrap(),
            postgres_port
        );

        let clickhouse_url = format!(
            "http://{}:{}",
            clickhouse_container.get_host().await.unwrap(),
            clickhouse_port
        );

        let postgres = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&postgres_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to PostgreSQL: {}", e))?;

        // Create ClickHouse client without database first
        let clickhouse_temp = Client::default()
            .with_url(&clickhouse_url)
            .with_user("default");

        // ================================ Run migrations ================================
        sqlx::migrate!("./src/db/pg/migrations")
            .run(&postgres)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to run PostgreSQL migrations: {}", e))?;

        // Initialize ClickHouse schema (creates database)
        Self::setup_clickhouse_schema(&clickhouse_temp).await?;

        // Now create client with the database set
        let clickhouse = Client::default()
            .with_url(&clickhouse_url)
            .with_user("default")
            .with_database("exchange");

        // ================================ Return database connections ================================
        let db = Db {
            postgres,
            clickhouse,
        };

        Ok(TestDb {
            db,
            postgres_container,
            clickhouse_container,
        })
    }

    /// Set up ClickHouse schema for testing using schema.sql file
    async fn setup_clickhouse_schema(client: &Client) -> anyhow::Result<()> {
        // Create database first
        client
            .query("CREATE DATABASE IF NOT EXISTS exchange")
            .execute()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create ClickHouse database: {}", e))?;

        // Read and execute schema from schema.sql file
        let schema_path = std::path::Path::new("src/db/ch/schema.sql");
        let schema_sql = std::fs::read_to_string(schema_path)
            .map_err(|e| anyhow::anyhow!("Failed to read ClickHouse schema file: {}", e))?;

        // Execute the schema SQL
        client
            .query(&schema_sql)
            .execute()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute ClickHouse schema: {}", e))?;

        Ok(())
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
        // Create tokens first
        self.create_test_token(base_ticker, 18, &format!("{} Token", base_ticker))
            .await?;
        self.create_test_token(quote_ticker, 18, &format!("{} Token", quote_ticker))
            .await?;

        // Then create the market
        self.create_test_market(base_ticker, quote_ticker).await
    }

    /// Helper function to create test candle
    pub async fn create_test_candle(
        self: &TestDb,
        market_id: &str,
        timestamp: chrono::DateTime<chrono::Utc>,
        ohlcv: (u128, u128, u128, u128, u128), // (open, high, low, close, volume)
    ) -> anyhow::Result<()> {
        self.db
            .insert_candle(market_id.to_string(), timestamp, ohlcv)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create test candle: {}", e))
    }
}

#[allow(dead_code)]
pub struct TestServer {
    pub address: String,
    pub db: Db,
    test_db: TestDb,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

#[allow(dead_code)]
impl TestServer {
    /// Start a test HTTP server on a random available port
    pub async fn start() -> anyhow::Result<Self> {
        // main.rs that needs to get the url and connect to db
        // instead, we can acess db handles directly! noice!
        let test_db = TestDb::setup().await?;

        // setup matching engine & api
        // exact same logic as main.rs
        let (engine_tx, engine_rx) = mpsc::channel::<EngineRequest>(100);
        let (event_tx, _) = broadcast::channel::<EngineEvent>(1000);
        let engine = MatchingEngine::new(test_db.db.clone(), engine_rx, event_tx.clone());
        tokio::spawn(async move {
            engine.run().await;
        });

        let rest = rest::create_rest();
        let ws = ws::create_ws();
        let state = AppState {
            db: test_db.db.clone(),
            engine_tx,
            event_tx,
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
            db: test_db.db.clone(),
            test_db,
            shutdown_tx,
        })
    }

    /// Helper to make a GET request
    pub async fn get(&self, path: &str) -> reqwest::Response {
        reqwest::get(&format!("{}{}", self.address, path))
            .await
            .expect("Failed to make GET request")
    }

    /// Helper to get a reqwest client for more complex requests
    pub fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

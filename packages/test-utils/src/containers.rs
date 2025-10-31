use backend::db::Db;
use std::env;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::{clickhouse::ClickHouse, postgres::Postgres};

/// Container handles for cleanup
pub struct TestContainers {
    pub(crate) db: Db,
    pub(crate) _postgres_container: testcontainers::ContainerAsync<Postgres>,
    pub(crate) _clickhouse_container: testcontainers::ContainerAsync<ClickHouse>,
}

impl TestContainers {
    /// Set up test databases with containers
    ///
    /// This starts PostgreSQL and ClickHouse containers, sets environment variables,
    /// and uses Db::connect() to automatically run migrations.
    /// The containers will be cleaned up when dropped.
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

        // ================================ Set environment variables ================================
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

        env::set_var("PG_URL", &postgres_url);
        env::set_var("CH_URL", &clickhouse_url);

        // ================================ Connect using Db::connect() ================================
        // This automatically runs migrations and creates schemas!
        let db = Db::connect()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to databases: {}", e))?;

        Ok(TestContainers {
            db,
            _postgres_container: postgres_container,
            _clickhouse_container: clickhouse_container,
        })
    }

    /// Get a clone of the database connection
    ///
    /// This is public to allow backend tests to access the database.
    /// Backend tests wrap this in their own TestDb struct.
    /// SDK tests should NOT use this - they should only test via HTTP API.
    pub fn db_clone(&self) -> Db {
        self.db.clone()
    }
}

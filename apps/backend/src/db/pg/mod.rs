use anyhow::Context;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

/// Create a PostgreSQL connection pool and run migrations
pub async fn create_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("PG_URL").context("PG_URL must be set in environment")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    // Run migrations automatically
    log::info!("Running PostgreSQL migrations...");
    sqlx::migrate!("./src/db/pg/migrations")
        .run(&pool)
        .await
        .context("Failed to run PostgreSQL migrations")?;
    log::info!("✅ PostgreSQL migrations complete");

    Ok(pool)
}

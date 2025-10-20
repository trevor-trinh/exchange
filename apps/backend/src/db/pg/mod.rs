use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

/// Create a PostgreSQL connection pool
pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("PG_URL").expect("PG_URL must be set in environment");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
}

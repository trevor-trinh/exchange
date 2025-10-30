use anyhow::Context;
use clickhouse::Client;
use std::env;

/// Create a ClickHouse client and initialize schema
pub async fn create_client() -> anyhow::Result<Client> {
    let clickhouse_url = env::var("CH_URL").context("CH_URL must be set in environment")?;

    // Create client without database first
    let client_no_db = Client::default()
        .with_url(&clickhouse_url)
        .with_user("default")
        .with_password("password");

    // Create database if it doesn't exist
    log::info!("Creating ClickHouse database...");
    client_no_db
        .query("CREATE DATABASE IF NOT EXISTS exchange")
        .execute()
        .await
        .context("Failed to create ClickHouse database")?;

    // Create client with database
    let client = Client::default()
        .with_url(&clickhouse_url)
        .with_user("default")
        .with_password("password")
        .with_database("exchange");

    // Run schema initialization
    log::info!("Running ClickHouse schema initialization...");
    init_schema(&client).await?;
    log::info!("✅ ClickHouse schema initialization complete");

    Ok(client)
}

/// Initialize ClickHouse schema (tables and materialized views)
async fn init_schema(client: &Client) -> anyhow::Result<()> {
    let schema = include_str!("schema.sql");

    // Split by statements and execute each
    for statement in schema.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            client
                .query(trimmed)
                .execute()
                .await
                .with_context(|| format!("Failed to execute: {}", trimmed))?;
        }
    }

    Ok(())
}

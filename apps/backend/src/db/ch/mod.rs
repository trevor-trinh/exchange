use anyhow::Context;
use clickhouse::Client;
use std::env;

/// Create a ClickHouse client and initialize schema
pub async fn create_client() -> anyhow::Result<Client> {
    let clickhouse_url = env::var("CH_URL").context("CH_URL must be set in environment")?;

    // Get credentials from env (empty password for testcontainers)
    let user = env::var("CH_USER").unwrap_or_else(|_| "default".to_string());
    let password = env::var("CH_PASSWORD").unwrap_or_else(|_| "".to_string());

    let mut client = Client::default().with_url(&clickhouse_url).with_user(&user);

    if !password.is_empty() {
        client = client.with_password(&password);
    }

    // Create database if it doesn't exist
    log::info!("Creating ClickHouse database...");
    client
        .query("CREATE DATABASE IF NOT EXISTS exchange")
        .execute()
        .await
        .context("Failed to create ClickHouse database")?;

    // Create client with database
    let mut client = Client::default()
        .with_url(&clickhouse_url)
        .with_user(&user)
        .with_database("exchange");

    if !password.is_empty() {
        client = client.with_password(&password);
    }

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

use anyhow::Context;
use clickhouse::Client;
use std::env;

/// Create a ClickHouse client and initialize schema
/// If url is provided, it will be used instead of reading from environment
pub async fn create_client(url: Option<String>) -> anyhow::Result<Client> {
    let clickhouse_url = url
        .or_else(|| env::var("CH_URL").ok())
        .context("CH_URL must be provided or set in environment")?;

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
        // Remove comment lines and inline comments
        let cleaned: String = statement
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                // Skip empty lines and lines that start with --
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    return None;
                }
                // Remove inline comments (everything after --)
                let without_inline_comment = if let Some(pos) = line.find("--") {
                    &line[..pos]
                } else {
                    line
                };
                Some(without_inline_comment)
            })
            .collect::<Vec<&str>>()
            .join("\n");

        let trimmed = cleaned.trim();
        if !trimmed.is_empty() {
            client
                .query(trimmed)
                .execute()
                .await
                .with_context(|| format!("Failed to execute: {}", trimmed))?;
        }
    }

    Ok(())
}

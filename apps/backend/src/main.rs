use anyhow::Context;
use backend::api::rest;
use backend::db::Db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::from_filename(".env.defaults").context("Failed to load .env.defaults")?;
    dotenvy::dotenv().context("Failed to load .env")?;

    env_logger::init();

    // Server configuration
    let host = std::env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let addr = format!("{}:{}", host, port);

    // Connect to databases
    let db = Db::connect()
        .await
        .context("Failed to connect to databases")?;
    log::info!("Connected to PostgreSQL and ClickHouse");

    // Create app with database state
    let app = rest::create_app(db);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind to {}", addr))?;

    println!("🚀 Backend server running on http://{}", addr);
    println!("📖 OpenAPI docs: http://{}/api/docs", addr);
    println!("📋 OpenAPI spec: http://{}/api/openapi.json", addr);

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}

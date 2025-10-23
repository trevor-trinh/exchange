use anyhow::Context;
use backend::api::rest;
use backend::db::Db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables: .env.defaults first, then .env overrides
    dotenvy::from_filename(".env.defaults")
        .context("Failed to load .env.defaults - file may be missing")?;
    dotenvy::dotenv().context("Failed to load .env file")?;

    // Initialize logging
    env_logger::init();

    // Get configuration from environment
    let host = std::env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let addr = format!("{}:{}", host, port);

    // Connect to databases
    let db = Db::connect()
        .await
        .context("Failed to connect to databases")?;

    log::info!("Connected to PostgreSQL and ClickHouse");

    // Verify database connection by running a simple query
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&db.postgres)
        .await
        .context("Failed to query database - ensure migrations have been run")?;

    log::info!(
        "Database connection verified - {} users in database",
        user_count.0
    );

    // Create and start the application with database state
    let app = rest::create_app(db);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind to address {}", addr))?;

    println!("Backend server running on http://{}", addr);
    println!("OpenAPI JSON available at http://{}/api/openapi.json", addr);
    println!("Swagger UI available at http://{}/api/docs", addr);

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}

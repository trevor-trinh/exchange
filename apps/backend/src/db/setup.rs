use std::env;
use std::process;

#[tokio::main]
async fn main() {
    // Load environment variables: .env.defaults first, then .env overrides
    dotenvy::from_filename(".env.defaults").ok();
    dotenvy::dotenv().ok();

    env_logger::init();

    if let Err(e) = run_setup().await {
        eprintln!("Database setup failed: {}", e);
        process::exit(1);
    }

    println!("✅ Database setup complete!");
}

async fn run_setup() -> Result<(), Box<dyn std::error::Error>> {
    // Setup Postgres
    setup_postgres().await?;

    // Setup ClickHouse
    setup_clickhouse().await?;

    Ok(())
}

async fn setup_postgres() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Setting up Postgres...");

    let database_url = env::var("PG_URL")
        .expect("PG_URL must be set");

    // Connect to Postgres
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./src/db/postgres/migrations")
        .run(&pool)
        .await?;

    println!("✅ Postgres setup complete");
    Ok(())
}

async fn setup_clickhouse() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Setting up ClickHouse...");

    let clickhouse_url = env::var("CH_URL")
        .expect("CH_URL must be set");

    // Parse ClickHouse connection details with credentials
    let client = clickhouse::Client::default()
        .with_url(&clickhouse_url)
        .with_user("default")
        .with_password("password");

    // Create database if not exists
    client
        .query("CREATE DATABASE IF NOT EXISTS exchange")
        .execute()
        .await?;

    // Read and execute schema file
    let schema_sql = include_str!("clickhouse/schema.sql");

    // Execute schema statements in the exchange database
    let client_with_db = client.with_database("exchange");
    client_with_db
        .query(schema_sql)
        .execute()
        .await?;

    println!("✅ ClickHouse setup complete");
    Ok(())
}

use backend::create_app;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    // Load environment variables: .env.defaults first, then .env overrides
    dotenvy::from_filename(".env.defaults").ok();
    dotenvy::dotenv().ok();

    // Initialize logging
    env_logger::init();

    // Get configuration from environment
    let host = std::env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let addr = format!("{}:{}", host, port);

    let app = create_app().await;

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Backend server running on http://{}:{}", host, port);
    println!(
        "OpenAPI JSON available at http://{}:{}/api/openapi.json",
        host, port
    );
    println!("Swagger UI available at http://{}:{}/api/docs", host, port);
    println!(
        "Postgres URL: {}",
        std::env::var("PG_URL").unwrap_or_else(|_| "PG_URL not set".to_string())
    );
    println!(
        "Clickhouse URL: {}",
        std::env::var("CH_URL").unwrap_or_else(|_| "CH_URL not set".to_string())
    );

    let pool = PgPoolOptions::new().max_connections(10).connect(&std::env::var("PG_URL").unwrap()).await.unwrap();

    let row = sqlx::query("SELECT * from users limit 1").fetch_one(&pool).await.unwrap();

    println!("Database connection successful: {:?}", row);
    axum::serve(listener, app).await.unwrap();

}

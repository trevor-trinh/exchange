use backend::create_app;

#[tokio::main]
async fn main() {
    // Load environment variables: .env.defaults first, then .env overrides
    dotenvy::from_filename(".env.defaults").ok();
    dotenvy::dotenv().ok();

    // Initialize logging
    env_logger::init();

    // Get configuration from environment
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = "8888".to_string();
    let addr = format!("{}:{}", host, port);

    let app = create_app().await;

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Backend server running on http://{}:{}", host, port);
    println!("OpenAPI JSON available at http://{}:{}/api/openapi.json", host, port);
    println!("Swagger UI available at http://{}:{}/api/docs", host, port);
    println!("Postgres URL: {}", std::env::var("PG_URL").unwrap_or_else(|_| "PG_URL not set".to_string()));
    println!("Clickhouse URL: {}", std::env::var("CH_URL").unwrap_or_else(|_| "CH_URL not set".to_string()));

    axum::serve(listener, app).await.unwrap();
}

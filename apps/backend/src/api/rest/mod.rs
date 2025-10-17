use axum::{Router, routing::get};

pub mod health;

pub fn create_routes() -> Router {
    Router::new().route("/api/health", get(health::health_check))
}

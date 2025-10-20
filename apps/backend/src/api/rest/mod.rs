use axum::{routing::get, Router};

pub mod health;

pub fn create_routes() -> Router {
    Router::new().route("/api/health", get(health::health_check))
}

// request -> handler -> db -> response
// /info
// - token info
// - market info

// /user
// - orders
// - balances
// - trades

// /trade - signature required
// request -> handler -> me -> response
// - order
// - cancel

// /drip - signature required
// - drip amount of tokens

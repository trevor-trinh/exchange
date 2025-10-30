//! Exchange SDK
//!
//! Rust SDK for interacting with the exchange API.
//!
//! This SDK provides:
//! - REST client for trading operations
//! - WebSocket client for real-time data
//! - Type-safe API using backend types
//!
//! # Example
//!
//! ```no_run
//! use exchange_sdk::ExchangeClient;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = ExchangeClient::new("http://localhost:8001");
//!     let markets = client.get_markets().await.unwrap();
//!     println!("Markets: {:?}", markets);
//! }
//! ```

pub mod client;
pub mod error;
pub mod websocket;

pub use client::ExchangeClient;
pub use error::{SdkError, SdkResult};
pub use websocket::{WebSocketClient, WebSocketHandle};

// Re-export backend types for convenience
pub use backend::models::api::{
    ApiCandle, CandlesRequest, CandlesResponse, ClientMessage, OrderCancelled,
    SubscriptionChannel,
};
pub use backend::models::domain::*;

/// SDK-specific OrderPlaced with domain types (pure u128)
#[derive(Debug, Clone)]
pub struct OrderPlaced {
    pub order: Order,
    pub trades: Vec<Trade>,
}

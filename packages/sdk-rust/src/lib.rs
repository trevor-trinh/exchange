//! Exchange SDK
//!
//! Rust SDK for interacting with the exchange API.
//!
//! This SDK provides:
//! - REST client for trading operations
//! - WebSocket client for real-time data
//! - Type-safe API using backend types
//! - Caching for markets and tokens
//! - Enhancement service for display values
//! - Formatting utilities
//! - Configurable logging
//!
//! # Example
//!
//! ```no_run
//! use exchange_sdk::ExchangeClient;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = ExchangeClient::new("http://localhost:8001");
//!
//!     // Get all markets
//!     let markets = client.get_markets().await.unwrap();
//!
//!     // Get specific market details
//!     let market = client.get_market("BTC/USDC").await.unwrap();
//!     println!("Market: {}/{}", market.base_ticker, market.quote_ticker);
//!
//!     // Get user trades
//!     let trades = client.get_trades("alice", Some("BTC/USDC".to_string())).await.unwrap();
//!     for trade in trades {
//!         println!("Trade: {} @ {}", trade.size, trade.price);
//!     }
//! }
//! ```

pub mod cache;
pub mod client;
pub mod enhancement;
pub mod error;
pub mod format;
pub mod logger;
pub mod websocket;

pub use cache::{CacheService, CacheStats};
pub use client::ExchangeClient;
pub use enhancement::{
    EnhancedBalance, EnhancedOrder, EnhancedOrderbookLevel, EnhancedTrade, EnhancementService,
};
pub use error::{SdkError, SdkResult};
pub use format::{format_number, format_price, format_size, to_atoms, to_display_value};
pub use logger::{ConsoleLogger, LogLevel, Logger, NoopLogger};
pub use websocket::{WebSocketClient, WebSocketHandle};

// Re-export backend types for convenience
pub use backend::models::api::{
    ApiCandle, CandlesRequest, CandlesResponse, ClientMessage, OrderCancelled, SubscriptionChannel,
};
pub use backend::models::domain::*;

/// SDK-specific OrderPlaced with domain types (pure u128)
#[derive(Debug, Clone)]
pub struct OrderPlaced {
    pub order: Order,
    pub trades: Vec<Trade>,
}

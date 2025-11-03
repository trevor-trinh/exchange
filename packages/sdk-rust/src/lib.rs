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
//! use exchange_sdk::{ExchangeClient, logger::{ConsoleLogger, LogLevel}};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let logger = Arc::new(ConsoleLogger::new(LogLevel::Info));
//!     let client = ExchangeClient::new("http://localhost:8001", logger);
//!
//!     // REST API automatically populates cache
//!     let markets = client.get_markets().await.unwrap();
//!
//!     // Access cached data
//!     let market = client.cache.get_market("BTC/USDC");
//!
//!     // Get enhanced data with display values
//!     let trades = client.get_trades("BTC/USDC", None).await.unwrap();
//!     for trade in trades {
//!         let enhanced = client.enhancer.enhance_trade(trade).unwrap();
//!         println!("Price: {}", enhanced.price_display);
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

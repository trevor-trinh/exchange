mod hyperliquid;
mod bots;

use anyhow::Result;
use bots::{OrderbookMirrorBot, OrderbookMirrorConfig, TradeMirrorBot, TradeMirrorConfig};
use exchange_sdk::ExchangeClient;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Exchange Bots");

    // Configuration
    let exchange_url = std::env::var("EXCHANGE_URL")
        .unwrap_or_else(|_| "http://localhost:8001".to_string());
    let coin = std::env::var("COIN")
        .unwrap_or_else(|_| "BTC".to_string());
    let market_id = std::env::var("MARKET_ID")
        .unwrap_or_else(|_| "BTC/USDC".to_string());
    let maker_address = std::env::var("MAKER_ADDRESS")
        .unwrap_or_else(|_| "maker_bot".to_string());
    let taker_address = std::env::var("TAKER_ADDRESS")
        .unwrap_or_else(|_| "taker_bot".to_string());

    info!("Exchange URL: {}", exchange_url);
    info!("Hyperliquid Coin: {}", coin);
    info!("Market ID: {}", market_id);

    // Create exchange clients
    let maker_client = ExchangeClient::new(&exchange_url);
    let taker_client = ExchangeClient::new(&exchange_url);

    // Configure orderbook mirror bot
    let orderbook_config = OrderbookMirrorConfig {
        coin: coin.clone(),
        market_id: market_id.clone(),
        user_address: maker_address,
        depth_levels: 5, // Mirror top 5 levels
        update_interval_ms: 1000,
        size_multiplier: Decimal::from_str("0.1").unwrap(), // 10% of Hyperliquid size
    };

    // Configure trade mirror bot
    let trade_config = TradeMirrorConfig {
        coin: coin.clone(),
        market_id: market_id.clone(),
        user_address: taker_address,
        size_multiplier: Decimal::from_str("0.1").unwrap(), // 10% of Hyperliquid size
        min_trade_size: Decimal::from_str("0.001").unwrap(), // Minimum 0.001 BTC
    };

    // Start bots in parallel
    let mut orderbook_bot = OrderbookMirrorBot::new(orderbook_config, maker_client);
    let mut trade_bot = TradeMirrorBot::new(trade_config, taker_client);

    let orderbook_handle = tokio::spawn(async move {
        if let Err(e) = orderbook_bot.start().await {
            tracing::error!("Orderbook bot error: {}", e);
        }
    });

    let trade_handle = tokio::spawn(async move {
        if let Err(e) = trade_bot.start().await {
            tracing::error!("Trade bot error: {}", e);
        }
    });

    // Wait for both bots
    tokio::select! {
        _ = orderbook_handle => {
            info!("Orderbook bot stopped");
        }
        _ = trade_handle => {
            info!("Trade bot stopped");
        }
    }

    Ok(())
}

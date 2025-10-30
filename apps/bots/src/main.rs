mod bots;
mod config;
mod hyperliquid;

use anyhow::{Context, Result};
use bots::{OrderbookMirrorBot, OrderbookMirrorConfig, TradeMirrorBot, TradeMirrorConfig};
use config::Config;
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

    info!("🤖 Starting Exchange Bots");

    // Load configuration
    let config = Config::load().context("Failed to load apps/bots/config.toml")?;

    // Exchange URL - can be overridden by env var
    let exchange_url =
        std::env::var("EXCHANGE_URL").unwrap_or_else(|_| config.exchange.url.clone());

    info!("📡 Exchange URL: {}", exchange_url);
    info!("👤 Maker address: {}", config.accounts.maker_address);
    info!("👤 Taker address: {}", config.accounts.taker_address);

    // Create exchange clients
    let maker_client = ExchangeClient::new(&exchange_url);
    let taker_client = ExchangeClient::new(&exchange_url);

    // Fund bot accounts if needed
    info!("💰 Funding bot accounts...");
    for token in ["BTC", "USDC"] {
        let amount = if token == "BTC" {
            &config.funding.btc_amount
        } else {
            &config.funding.usdc_amount
        };

        // Try to fund maker bot
        if let Err(e) = maker_client
            .admin_faucet(
                config.accounts.maker_address.clone(),
                token.to_string(),
                amount.clone(),
            )
            .await
        {
            // Ignore errors - account might already be funded
            tracing::debug!(
                "Maker funding for {} (ignoring if already funded): {}",
                token,
                e
            );
        }

        // Try to fund taker bot
        if let Err(e) = taker_client
            .admin_faucet(
                config.accounts.taker_address.clone(),
                token.to_string(),
                amount.clone(),
            )
            .await
        {
            tracing::debug!(
                "Taker funding for {} (ignoring if already funded): {}",
                token,
                e
            );
        }
    }
    info!("✓ Bot accounts funded");

    // Configure orderbook mirror bot (if enabled)
    let orderbook_config = if config.orderbook_mirror.enabled {
        Some(OrderbookMirrorConfig {
            coin: config.orderbook_mirror.coin.clone(),
            market_id: config.orderbook_mirror.market_id.clone(),
            user_address: config.accounts.maker_address.clone(),
            depth_levels: config.orderbook_mirror.depth_levels,
            update_interval_ms: config.orderbook_mirror.update_interval_ms,
            size_multiplier: Decimal::from_str(&config.orderbook_mirror.size_multiplier)
                .context("Invalid size_multiplier")?,
        })
    } else {
        None
    };

    // Configure trade mirror bot (if enabled)
    let trade_config = if config.trade_mirror.enabled {
        Some(TradeMirrorConfig {
            coin: config.trade_mirror.coin.clone(),
            market_id: config.trade_mirror.market_id.clone(),
            user_address: config.accounts.taker_address.clone(),
            size_multiplier: Decimal::from_str(&config.trade_mirror.size_multiplier)
                .context("Invalid size_multiplier")?,
            min_trade_size: Decimal::from_str(&config.trade_mirror.min_trade_size)
                .context("Invalid min_trade_size")?,
        })
    } else {
        None
    };

    // Start bots in parallel
    let mut handles = vec![];

    // Start orderbook mirror bot if enabled
    if let Some(config) = orderbook_config {
        info!("📖 Starting orderbook mirror bot for {}", config.market_id);
        let mut orderbook_bot = OrderbookMirrorBot::new(config, maker_client);

        let handle = tokio::spawn(async move {
            if let Err(e) = orderbook_bot.start().await {
                tracing::error!("❌ Orderbook bot error: {}", e);
            }
        });
        handles.push(handle);
    }

    // Start trade mirror bot if enabled
    if let Some(config) = trade_config {
        info!("💱 Starting trade mirror bot for {}", config.market_id);
        let mut trade_bot = TradeMirrorBot::new(config, taker_client);

        let handle = tokio::spawn(async move {
            if let Err(e) = trade_bot.start().await {
                tracing::error!("❌ Trade bot error: {}", e);
            }
        });
        handles.push(handle);
    }

    if handles.is_empty() {
        info!("❌ No bots enabled in config.toml");
        return Ok(());
    }

    info!("✅ All enabled bots are running");

    // Wait for any bot to finish
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

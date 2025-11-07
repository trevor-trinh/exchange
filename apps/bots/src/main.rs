mod config;
mod markets;
mod utils;

use anyhow::{Context, Result};
use config::Config;
use exchange_sdk::ExchangeClient;
use markets::bp_usdc::{LmsrConfig, LmsrMarketMakerBot, SyntheticTraderBot, SyntheticTraderConfig};
use markets::btc_usdc::{
    OrderbookMirrorBot, OrderbookMirrorConfig, TradeMirrorBot, TradeMirrorConfig,
};
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

    // Start bots in parallel
    let mut handles = vec![];

    // ===========================
    // BTC/USDC Market Bots
    // ===========================

    if let Some(btc_config) = &config.markets.btc_usdc {
        if btc_config.enabled {
            info!("🟡 BTC/USDC market enabled");

            // Orderbook mirror bot
            if let Some(ob_config) = &btc_config.orderbook_mirror {
                if ob_config.enabled {
                    let bot_config = OrderbookMirrorConfig {
                        market_id: "BTC/USDC".to_string(),
                        user_address: ob_config.user_address.clone(),
                        depth_levels: ob_config.depth_levels,
                        update_interval_ms: ob_config.update_interval_ms,
                    };

                    info!("📖 Initializing orderbook mirror bot for BTC/USDC");
                    let client = ExchangeClient::new(&exchange_url);
                    let mut bot = OrderbookMirrorBot::new(bot_config, client)
                        .await
                        .context("Failed to initialize orderbook mirror bot")?;

                    let handle = tokio::spawn(async move {
                        if let Err(e) = bot.start().await {
                            tracing::error!("❌ Orderbook bot error: {}", e);
                        }
                    });
                    handles.push(handle);
                }
            }

            // Trade mirror bot
            if let Some(tm_config) = &btc_config.trade_mirror {
                if tm_config.enabled {
                    let bot_config = TradeMirrorConfig {
                        market_id: "BTC/USDC".to_string(),
                        user_address: tm_config.user_address.clone(),
                    };

                    info!("💱 Initializing trade mirror bot for BTC/USDC");
                    let client = ExchangeClient::new(&exchange_url);
                    let mut bot = TradeMirrorBot::new(bot_config, client)
                        .await
                        .context("Failed to initialize trade mirror bot")?;

                    let handle = tokio::spawn(async move {
                        if let Err(e) = bot.start().await {
                            tracing::error!("❌ Trade bot error: {}", e);
                        }
                    });
                    handles.push(handle);
                }
            }
        }
    }

    // ===========================
    // BP/USDC Market Bots
    // ===========================

    if let Some(bp_config) = &config.markets.bp_usdc {
        if bp_config.enabled {
            info!("🔵 BP/USDC market enabled");

            // LMSR market maker bot
            if let Some(lmsr_config) = &bp_config.lmsr {
                if lmsr_config.enabled {
                    let bot_config = LmsrConfig {
                        user_address: lmsr_config.user_address.clone(),
                        liquidity_param: lmsr_config.liquidity_param,
                        initial_probability: lmsr_config.initial_probability,
                        update_interval_ms: lmsr_config.update_interval_ms,
                        spread_bps: lmsr_config.spread_bps,
                    };

                    info!("📊 Initializing LMSR market maker for BP/USDC");
                    let client = ExchangeClient::new(&exchange_url);
                    let mut bot = LmsrMarketMakerBot::new(bot_config, client)
                        .await
                        .context("Failed to initialize LMSR market maker")?;

                    let handle = tokio::spawn(async move {
                        if let Err(e) = bot.start().await {
                            tracing::error!("❌ LMSR bot error: {}", e);
                        }
                    });
                    handles.push(handle);
                }
            }

            // Synthetic trader bot
            if let Some(trader_config) = &bp_config.synthetic_trader {
                if trader_config.enabled {
                    let bot_config = SyntheticTraderConfig {
                        user_address: trader_config.user_address.clone(),
                        min_interval_ms: trader_config.min_interval_ms,
                        max_interval_ms: trader_config.max_interval_ms,
                        min_size: trader_config.min_size,
                        max_size: trader_config.max_size,
                        buy_probability: trader_config.buy_probability,
                    };

                    info!("🎲 Initializing synthetic trader for BP/USDC");
                    let client = ExchangeClient::new(&exchange_url);
                    let mut bot = SyntheticTraderBot::new(bot_config, client)
                        .await
                        .context("Failed to initialize synthetic trader")?;

                    let handle = tokio::spawn(async move {
                        if let Err(e) = bot.start().await {
                            tracing::error!("❌ Synthetic trader error: {}", e);
                        }
                    });
                    handles.push(handle);
                }
            }
        }
    }

    // ===========================
    // Run all bots
    // ===========================

    if handles.is_empty() {
        info!("❌ No bots enabled in config.toml");
        return Ok(());
    }

    info!("✅ All enabled bots are running ({} total)", handles.len());

    // Wait for any bot to finish
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

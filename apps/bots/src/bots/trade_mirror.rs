use crate::hyperliquid::{HyperliquidClient, HlMessage};
use anyhow::Result;
use backend::models::domain::{OrderType, Side};
use exchange_sdk::ExchangeClient;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::str::FromStr;
use tracing::{error, info, warn};

/// Configuration for the trade mirror bot
#[derive(Clone)]
pub struct TradeMirrorConfig {
    pub coin: String,                   // e.g., "BTC"
    pub market_id: String,              // e.g., "BTC/USDC"
    pub user_address: String,           // Bot's wallet address
    pub size_multiplier: Decimal,       // Scale trade sizes (e.g., 0.1 for 10% of Hyperliquid)
    pub min_trade_size: Decimal,        // Minimum trade size to mirror
}

/// Trade mirror bot - creates realistic trading activity by copying Binance trades
pub struct TradeMirrorBot {
    config: TradeMirrorConfig,
    exchange_client: ExchangeClient,
}

impl TradeMirrorBot {
    pub fn new(config: TradeMirrorConfig, exchange_client: ExchangeClient) -> Self {
        Self {
            config,
            exchange_client,
        }
    }

    /// Start the bot
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting trade mirror bot for {}", self.config.coin);

        // Connect to Hyperliquid
        let hl_client = HyperliquidClient::new(self.config.coin.clone());

        let (mut rx, _handle) = hl_client.start().await?;

        // Process messages
        while let Some(msg) = rx.recv().await {
            match msg {
                HlMessage::L2Book(_) => {
                    // Trade bot doesn't care about orderbook
                }
                HlMessage::Trade(trades) => {
                    // Mirror each trade
                    for trade in trades {
                        if let Err(e) = self.mirror_trade(&trade.px, &trade.sz, &trade.side).await {
                            error!("Failed to mirror trade: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Mirror a Hyperliquid trade on our exchange
    async fn mirror_trade(
        &self,
        price_str: &str,
        size_str: &str,
        side_str: &str,
    ) -> Result<()> {
        // Parse trade details
        let size = Decimal::from_str(size_str)?;

        // Apply size multiplier
        let adjusted_size = size * self.config.size_multiplier;

        // Skip tiny trades
        if adjusted_size < self.config.min_trade_size {
            return Ok(());
        }

        // Convert to our exchange format
        let price = self.convert_price_from_str(price_str)?;
        let size_formatted = self.convert_size(adjusted_size);

        // Determine side from Hyperliquid:
        // "A" = ask/sell (seller initiated), "B" = bid/buy (buyer initiated)
        let side = match side_str {
            "A" => Side::Sell,
            "B" => Side::Buy,
            _ => {
                warn!("Unknown trade side: {}", side_str);
                return Ok(());
            }
        };

        info!(
            "Mirroring {} trade: {:?} {} @ {}",
            self.config.coin, side, size_formatted, price
        );

        // Place market order to execute the trade
        match self
            .exchange_client
            .place_order(
                self.config.user_address.clone(),
                self.config.market_id.clone(),
                side,
                OrderType::Market,
                price,
                size_formatted,
                "trade_mirror".to_string(),
            )
            .await
        {
            Ok(result) => {
                info!("Trade mirrored successfully: {} trades executed", result.trades.len());
            }
            Err(e) => {
                warn!("Failed to place trade mirror order: {}", e);
            }
        }

        Ok(())
    }

    /// Convert Binance price string to our exchange format (u128 as string)
    fn convert_price_from_str(&self, price_str: &str) -> Result<String> {
        let price = Decimal::from_str(price_str)?;
        // Assuming 6 decimals for price precision
        let scaled = price * Decimal::from(1_000_000);
        Ok(scaled.to_u128().unwrap_or(0).to_string())
    }

    /// Convert Binance size to our exchange format
    fn convert_size(&self, size: Decimal) -> String {
        // Assuming 6 decimals for size precision
        let scaled = size * Decimal::from(1_000_000);
        scaled.to_u128().unwrap_or(0).to_string()
    }
}

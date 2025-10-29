use crate::hyperliquid::{HyperliquidClient, HlMessage, Orderbook};
use anyhow::Result;
use backend::models::domain::{OrderType, Side};
use exchange_sdk::ExchangeClient;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Configuration for the orderbook mirror bot
#[derive(Clone)]
pub struct OrderbookMirrorConfig {
    pub coin: String,                   // e.g., "BTC"
    pub market_id: String,              // e.g., "BTC/USDC"
    pub user_address: String,           // Bot's wallet address
    pub depth_levels: usize,            // How many levels to mirror (e.g., 5)
    pub update_interval_ms: u64,        // Min time between order updates
    pub size_multiplier: Decimal,       // Scale order sizes (e.g., 0.1 for 10% of Hyperliquid)
}

/// Orderbook mirror bot - maintains liquidity by copying Binance's orderbook
pub struct OrderbookMirrorBot {
    config: OrderbookMirrorConfig,
    exchange_client: ExchangeClient,
    orderbook: Orderbook,
    active_orders: HashMap<String, Uuid>, // price_side -> order_id
}

impl OrderbookMirrorBot {
    pub fn new(config: OrderbookMirrorConfig, exchange_client: ExchangeClient) -> Self {
        let orderbook = Orderbook::new(config.coin.clone());

        Self {
            config,
            exchange_client,
            orderbook,
            active_orders: HashMap::new(),
        }
    }

    /// Start the bot
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting orderbook mirror bot for {}", self.config.coin);

        // Connect to Hyperliquid
        let hl_client = HyperliquidClient::new(self.config.coin.clone());

        let (mut rx, _handle) = hl_client.start().await?;

        // Process messages
        while let Some(msg) = rx.recv().await {
            match msg {
                HlMessage::L2Book(book_data) => {
                    // Update orderbook from Hyperliquid L2 data
                    if book_data.levels.len() >= 2 {
                        let bids = book_data.levels[0].clone();
                        let asks = book_data.levels[1].clone();
                        self.orderbook.update_from_l2(bids, asks);

                        // Sync orders with exchange
                        if let Err(e) = self.sync_orderbook().await {
                            error!("Failed to sync orderbook: {}", e);
                        }
                    }
                }
                HlMessage::Trade(_) => {
                    // Orderbook bot doesn't care about trades
                }
            }
        }

        Ok(())
    }

    /// Sync our exchange's orderbook with Binance
    async fn sync_orderbook(&mut self) -> Result<()> {
        let (bids, asks) = self.orderbook.get_top_levels(self.config.depth_levels);

        // Cancel all existing orders
        self.cancel_all_orders().await?;

        // Place new bid orders
        for level in bids {
            let price = self.convert_price(level.price);
            let size = self.convert_size(level.quantity);

            match self
                .exchange_client
                .place_order(
                    self.config.user_address.clone(),
                    self.config.market_id.clone(),
                    Side::Buy,
                    OrderType::Limit,
                    price.clone(),
                    size.clone(),
                    "orderbook_mirror".to_string(),
                )
                .await
            {
                Ok(result) => {
                    let order_id = result.order.id;
                    let key = format!("{}_{}", price, "buy");
                    self.active_orders.insert(key, order_id);
                }
                Err(e) => {
                    warn!("Failed to place bid order at {}: {}", price, e);
                }
            }
        }

        // Place new ask orders
        for level in asks {
            let price = self.convert_price(level.price);
            let size = self.convert_size(level.quantity);

            match self
                .exchange_client
                .place_order(
                    self.config.user_address.clone(),
                    self.config.market_id.clone(),
                    Side::Sell,
                    OrderType::Limit,
                    price.clone(),
                    size.clone(),
                    "orderbook_mirror".to_string(),
                )
                .await
            {
                Ok(result) => {
                    let order_id = result.order.id;
                    let key = format!("{}_{}", price, "sell");
                    self.active_orders.insert(key, order_id);
                }
                Err(e) => {
                    warn!("Failed to place ask order at {}: {}", price, e);
                }
            }
        }

        Ok(())
    }

    /// Cancel all active orders
    async fn cancel_all_orders(&mut self) -> Result<()> {
        let order_ids: Vec<Uuid> = self.active_orders.values().copied().collect();

        for order_id in order_ids {
            match self
                .exchange_client
                .cancel_order(
                    self.config.user_address.clone(),
                    order_id.to_string(),
                    "orderbook_mirror".to_string(),
                )
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to cancel order {}: {}", order_id, e);
                }
            }
        }

        self.active_orders.clear();
        Ok(())
    }

    /// Convert Binance price to our exchange format (u128 as string)
    fn convert_price(&self, price: Decimal) -> String {
        // Assuming 6 decimals for price precision
        let scaled = price * Decimal::from(1_000_000);
        scaled.to_u128().unwrap_or(0).to_string()
    }

    /// Convert Binance size to our exchange format, applying multiplier
    fn convert_size(&self, size: Decimal) -> String {
        let adjusted = size * self.config.size_multiplier;
        // Assuming 6 decimals for size precision
        let scaled = adjusted * Decimal::from(1_000_000);
        scaled.to_u128().unwrap_or(0).to_string()
    }
}

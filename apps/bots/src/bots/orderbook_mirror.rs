use crate::hyperliquid::{HlMessage, HyperliquidClient, Orderbook};
use anyhow::Result;
use backend::models::domain::{Market, OrderType, Side};
use exchange_sdk::ExchangeClient;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Configuration for the orderbook mirror bot
#[derive(Clone)]
pub struct OrderbookMirrorConfig {
    pub market_id: String,       // e.g., "BTC/USDC"
    pub user_address: String,    // Bot's wallet address
    pub depth_levels: usize,     // How many levels to mirror (e.g., 5)
    pub update_interval_ms: u64, // Min time between order updates
}

/// Orderbook mirror bot - maintains liquidity by copying Hyperliquid's orderbook
pub struct OrderbookMirrorBot {
    config: OrderbookMirrorConfig,
    exchange_client: ExchangeClient,
    orderbook: Orderbook,
    active_orders: HashMap<String, Uuid>, // price_side -> order_id

    // Market configuration fetched from backend
    market: Market,
}

impl OrderbookMirrorBot {
    pub async fn new(
        config: OrderbookMirrorConfig,
        exchange_client: ExchangeClient,
    ) -> Result<Self> {
        // Fetch market configuration from backend
        let market = exchange_client.get_market(&config.market_id).await?;

        info!(
            "Orderbook mirror bot initialized for market {}",
            config.market_id
        );
        info!(
            "Market: {} (base) / {} (quote)",
            market.base_ticker, market.quote_ticker
        );

        // Auto-faucet initial funds (large amounts for testing)
        // Bots will auto-faucet more if they run out during operation
        info!(
            "💰 Auto-fauceting initial funds for {}",
            config.user_address
        );
        let faucet_amount = "10000000000000000000000000"; // Large amount for testing

        for token in [&market.base_ticker, &market.quote_ticker] {
            match exchange_client
                .admin_faucet(
                    config.user_address.clone(),
                    token.to_string(),
                    faucet_amount.to_string(),
                )
                .await
            {
                Ok(_) => info!("✓ Fauceted {} for {}", token, config.user_address),
                Err(e) => info!("Note: Faucet {} for {}: {}", token, config.user_address, e),
            }
        }

        // Use base ticker as coin symbol for Hyperliquid (e.g., "BTC" from "BTC/USDC")
        let coin = market.base_ticker.clone();
        let orderbook = Orderbook::new(coin.clone());

        Ok(Self {
            config,
            exchange_client,
            orderbook,
            active_orders: HashMap::new(),
            market,
        })
    }

    /// Start the bot
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting orderbook mirror bot for {} PERP -> {} market",
            self.market.base_ticker, self.config.market_id
        );
        info!(
            "Update interval: {}ms (throttling to prevent spam)",
            self.config.update_interval_ms
        );

        // Connect to Hyperliquid (perps by default)
        let hl_client = HyperliquidClient::new(self.market.base_ticker.clone());

        let (mut rx, _handle) = hl_client.start().await?;

        // Throttling: track last update time
        let mut last_sync = Instant::now();
        let update_interval = Duration::from_millis(self.config.update_interval_ms);

        // Process messages
        while let Some(msg) = rx.recv().await {
            match msg {
                HlMessage::L2Book(book_data) => {
                    // Update local orderbook snapshot (fast, in-memory)
                    if book_data.levels.len() >= 2 {
                        let bids = book_data.levels[0].clone();
                        let asks = book_data.levels[1].clone();
                        self.orderbook.update_from_l2(bids, asks);

                        // Only sync with exchange if enough time has passed (throttling)
                        let now = Instant::now();
                        if now.duration_since(last_sync) >= update_interval {
                            if let Err(e) = self.sync_orderbook().await {
                                error!("Failed to sync orderbook: {}", e);
                            }
                            last_sync = now;
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

    /// Sync our exchange's orderbook with Hyperliquid
    async fn sync_orderbook(&mut self) -> Result<()> {
        let (bids, asks) = self.orderbook.get_top_levels(self.config.depth_levels);

        // Store old orders before placing new ones
        // This ensures we maintain liquidity during the transition
        let old_orders = self.active_orders.clone();
        self.active_orders.clear();

        // Place new bid orders FIRST (before cancelling old ones)
        for level in bids {
            let price = level.price.to_string();
            let size = level.quantity.to_string();

            match self
                .exchange_client
                .place_order_decimal(
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
                    let err_msg = e.to_string();
                    warn!("Failed to place bid order at {}: {}", price, err_msg);

                    // Try to auto-faucet if it's a balance error
                    self.auto_faucet_on_error(&err_msg).await;
                }
            }
        }

        // Place new ask orders
        for level in asks {
            let price = level.price.to_string();
            let size = level.quantity.to_string();

            match self
                .exchange_client
                .place_order_decimal(
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
                    let err_msg = e.to_string();
                    warn!("Failed to place ask order at {}: {}", price, err_msg);

                    // Try to auto-faucet if it's a balance error
                    self.auto_faucet_on_error(&err_msg).await;
                }
            }
        }

        // Now cancel the old orders (after new ones are placed)
        self.cancel_old_orders(old_orders).await?;

        Ok(())
    }

    /// Cancel specific old orders (used after placing new orders)
    async fn cancel_old_orders(&self, old_orders: HashMap<String, Uuid>) -> Result<()> {
        if old_orders.is_empty() {
            return Ok(());
        }

        let mut cancelled_count = 0;
        for (_key, order_id) in old_orders {
            match self
                .exchange_client
                .cancel_order(
                    self.config.user_address.clone(),
                    order_id.to_string(),
                    "orderbook_mirror".to_string(),
                )
                .await
            {
                Ok(_) => {
                    cancelled_count += 1;
                }
                Err(e) => {
                    // It's okay if order is already filled/cancelled
                    warn!("Failed to cancel old order {}: {}", order_id, e);
                }
            }
        }

        if cancelled_count > 0 {
            info!("Cancelled {} old orders", cancelled_count);
        }

        Ok(())
    }

    /// Cancel all active orders
    async fn cancel_all_orders(&mut self) -> Result<()> {
        // Use the new cancel_all_orders endpoint for efficient bulk cancellation
        match self
            .exchange_client
            .cancel_all_orders(
                self.config.user_address.clone(),
                Some(self.config.market_id.clone()),
                "orderbook_mirror".to_string(),
            )
            .await
        {
            Ok(result) => {
                info!(
                    "Cancelled {} orders for market {}",
                    result.count, self.config.market_id
                );
            }
            Err(e) => {
                warn!("Failed to cancel all orders: {}", e);
            }
        }

        self.active_orders.clear();
        Ok(())
    }

    /// Auto-faucet funds if we detect insufficient balance error
    async fn auto_faucet_on_error(&self, error_msg: &str) -> bool {
        // Check if error is about insufficient balance
        if error_msg.contains("Insufficient balance") || error_msg.contains("insufficient") {
            // Extract token from error message if possible
            let token = if error_msg.contains("BTC") {
                Some("BTC")
            } else if error_msg.contains("USDC") {
                Some("USDC")
            } else {
                None
            };

            if let Some(token_name) = token {
                info!(
                    "💰 Detected insufficient {}, auto-fauceting more...",
                    token_name
                );
                let faucet_amount = "10000000000000000000000000";

                match self
                    .exchange_client
                    .admin_faucet(
                        self.config.user_address.clone(),
                        token_name.to_string(),
                        faucet_amount.to_string(),
                    )
                    .await
                {
                    Ok(_) => {
                        info!(
                            "✓ Auto-fauceted {} for {}",
                            token_name, self.config.user_address
                        );
                        return true;
                    }
                    Err(e) => {
                        warn!("Failed to auto-faucet {}: {}", token_name, e);
                    }
                }
            }
        }
        false
    }
}

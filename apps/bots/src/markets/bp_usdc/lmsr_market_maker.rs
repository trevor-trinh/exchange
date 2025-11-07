use crate::utils::bot_helpers;
use anyhow::Result;
use backend::models::domain::{Market, OrderType, Side};
use exchange_sdk::ExchangeClient;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Configuration for the LMSR market maker bot
#[derive(Clone, Debug)]
pub struct LmsrConfig {
    pub user_address: String,
    pub liquidity_param: f64,      // b parameter in LMSR
    pub initial_probability: f64,  // Starting probability [0, 1]
    pub update_interval_ms: u64,   // Quote update frequency
    pub spread_bps: u64,            // Spread in basis points (1 bps = 0.01%)
}

/// LMSR Market Maker bot - provides liquidity for prediction markets
///
/// Uses Logarithmic Market Scoring Rule (LMSR) for pricing:
/// - Price(outcome) = exp(q_outcome / b) / sum(exp(q_i / b))
/// - b = liquidity parameter (controls market depth)
/// - q_i = cumulative shares sold for outcome i
pub struct LmsrMarketMakerBot {
    config: LmsrConfig,
    exchange_client: ExchangeClient,
    market: Market,

    // LMSR state
    cumulative_shares_yes: f64,  // Total shares sold for YES outcome
    cumulative_shares_no: f64,   // Total shares sold for NO outcome

    // Order tracking
    active_orders: HashMap<String, Uuid>, // side -> order_id ("bid" or "ask")
    last_update: Instant,
}

impl LmsrMarketMakerBot {
    pub async fn new(
        config: LmsrConfig,
        exchange_client: ExchangeClient,
    ) -> Result<Self> {
        info!("LMSR Market Maker bot initialized for BP/USDC");

        // Fetch market configuration and auto-faucet initial funds
        let market = bot_helpers::fetch_market_and_faucet(
            &exchange_client,
            "BP/USDC",
            &config.user_address,
        )
        .await?;

        // Initialize cumulative shares based on initial probability
        // For p = 0.5, we want q_yes = q_no = 0
        // For p != 0.5, solve: p = exp(q_yes/b) / (exp(q_yes/b) + exp(q_no/b))
        let b = config.liquidity_param;
        let p = config.initial_probability;

        // Set q_no = 0 as reference point
        // Then q_yes = b * ln(p / (1 - p))
        let cumulative_shares_no = 0.0;
        let cumulative_shares_yes = if p > 0.0 && p < 1.0 {
            b * (p / (1.0 - p)).ln()
        } else {
            0.0
        };

        info!(
            "Initial state: p={:.3}, q_yes={:.2}, q_no={:.2}, b={}",
            p, cumulative_shares_yes, cumulative_shares_no, b
        );

        Ok(Self {
            config,
            exchange_client,
            market,
            cumulative_shares_yes,
            cumulative_shares_no,
            active_orders: HashMap::new(),
            last_update: Instant::now(),
        })
    }

    /// Start the bot
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting LMSR market maker for BP/USDC");
        info!(
            "Update interval: {}ms, Spread: {} bps",
            self.config.update_interval_ms, self.config.spread_bps
        );

        // Cancel all existing orders on startup
        info!("Cancelling any existing orders from previous runs...");
        self.cancel_all_orders().await?;

        // Place initial orders immediately
        info!("Placing initial orders...");
        if let Err(e) = self.update_quotes().await {
            error!("Error placing initial orders: {}", e);
        }
        self.last_update = Instant::now();

        loop {
            // Check if it's time to update
            if self.last_update.elapsed().as_millis() >= self.config.update_interval_ms as u128 {
                if let Err(e) = self.update_quotes().await {
                    error!("Error updating quotes: {}", e);
                }
                self.last_update = Instant::now();
            }

            // Sleep a bit to avoid busy loop
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Update quotes based on current LMSR state
    async fn update_quotes(&mut self) -> Result<()> {
        info!("→ update_quotes() called");
        // Calculate current LMSR price
        let lmsr_price = self.calculate_lmsr_price();
        info!("→ Calculated LMSR price: {:.4}", lmsr_price);

        // Add spread
        let spread = self.config.spread_bps as f64 / 10000.0; // Convert bps to decimal
        let mut bid_price = lmsr_price * (1.0 - spread);
        let mut ask_price = lmsr_price * (1.0 + spread);

        // Round to tick size (0.001 for BP/USDC prediction market)
        // This ensures prices are valid multiples of the tick size
        let tick_size = 0.001;
        bid_price = (bid_price / tick_size).floor() * tick_size;
        ask_price = (ask_price / tick_size).ceil() * tick_size;

        // Clamp prices to [0.001, 0.999] range (prediction market bounds)
        // Bid should be lower, ask should be higher
        bid_price = bid_price.max(0.001).min(0.998);
        ask_price = ask_price.max(0.002).min(0.999);

        // Ensure bid < ask (prevent crossed market)
        if bid_price >= ask_price {
            warn!("Crossed market detected! bid={:.4}, ask={:.4}, adjusting...", bid_price, ask_price);
            // Center around mid-point with minimum spread
            let mid = (bid_price + ask_price) / 2.0;
            let min_spread = 0.001; // Minimum 0.1% spread
            bid_price = (mid - min_spread).max(0.001);
            ask_price = (mid + min_spread).min(0.999);
        }

        info!(
            "LMSR price: {:.4}, Bid: {:.4}, Ask: {:.4}",
            lmsr_price, bid_price, ask_price
        );

        // Cancel existing orders
        info!("→ Cancelling existing orders...");
        self.cancel_all_orders().await?;
        info!("→ Orders cancelled");

        // Place new bid order (buying BP)
        let bid_size = 100.0; // Fixed size for now
        info!("→ Placing bid order at {:.4} for size {:.2}", bid_price, bid_size);
        if let Err(e) = self.place_order(Side::Buy, bid_price, bid_size).await {
            warn!("❌ Failed to place bid: {}", e);
            bot_helpers::auto_faucet_on_error(
                &self.exchange_client,
                &self.config.user_address,
                &self.market,
                &e.to_string(),
            )
            .await;
        }

        // Place new ask order (selling BP)
        let ask_size = 100.0; // Fixed size for now
        info!("→ Placing ask order at {:.4} for size {:.2}", ask_price, ask_size);
        if let Err(e) = self.place_order(Side::Sell, ask_price, ask_size).await {
            warn!("❌ Failed to place ask: {}", e);
            bot_helpers::auto_faucet_on_error(
                &self.exchange_client,
                &self.config.user_address,
                &self.market,
                &e.to_string(),
            )
            .await;
        }

        Ok(())
    }

    /// Calculate LMSR price for YES outcome
    /// Price = exp(q_yes / b) / (exp(q_yes / b) + exp(q_no / b))
    fn calculate_lmsr_price(&self) -> f64 {
        let b = self.config.liquidity_param;
        let exp_yes = (self.cumulative_shares_yes / b).exp();
        let exp_no = (self.cumulative_shares_no / b).exp();

        exp_yes / (exp_yes + exp_no)
    }

    /// Place an order
    async fn place_order(&mut self, side: Side, price: f64, size: f64) -> Result<()> {
        let result = self
            .exchange_client
            .place_order_decimal(
                self.config.user_address.clone(),
                "BP/USDC".to_string(),
                side.clone(),
                OrderType::Limit,
                format!("{:.6}", price),
                format!("{:.6}", size),
                "lmsr_market_maker".to_string(),
            )
            .await?;

        let key = match side {
            Side::Buy => "bid",
            Side::Sell => "ask",
        };
        self.active_orders.insert(key.to_string(), result.order.id);

        Ok(())
    }

    /// Cancel all active orders
    async fn cancel_all_orders(&mut self) -> Result<()> {
        if self.active_orders.is_empty() {
            return Ok(());
        }

        match self
            .exchange_client
            .cancel_all_orders(
                self.config.user_address.clone(),
                Some("BP/USDC".to_string()),
                "lmsr_market_maker".to_string(),
            )
            .await
        {
            Ok(result) => {
                if result.count > 0 {
                    info!("Cancelled {} orders", result.count);
                }
            }
            Err(e) => {
                warn!("Failed to cancel all orders: {}", e);
            }
        }

        self.active_orders.clear();
        Ok(())
    }

    // TODO: In the future, we would:
    // 1. Listen to fill events via WebSocket
    // 2. Update cumulative_shares_yes/no when fills occur
    // 3. This will cause the LMSR price to adjust automatically
    //
    // For now, the price stays constant unless manually adjusted
}

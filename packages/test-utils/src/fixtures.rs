use crate::TestServer;
use exchange_sdk::ExchangeClient;

/// High-level test fixture for exchange testing
///
/// Provides a running test exchange with a pre-configured market and admin client.
/// Use this for SDK and integration tests that need a complete exchange setup.
pub struct TestExchange {
    pub server: TestServer,
    pub client: ExchangeClient,
    pub market_id: String,
    pub base_ticker: String,
    pub quote_ticker: String,
    pub base_decimals: u32,
    pub quote_decimals: u32,
}

impl TestExchange {
    /// Create a new test exchange with BTC/USDC market (default: 6 decimals each)
    ///
    /// # Example - Using helper methods (recommended)
    /// ```rust,ignore
    /// let fixture = TestExchange::new().await?;
    ///
    /// // Use helper methods to convert human-readable amounts
    /// fixture.create_user_with_balance(
    ///     "alice",
    ///     fixture.to_base_atoms(10.0),    // 10 BTC
    ///     fixture.to_quote_atoms(50000.0) // 50,000 USDC
    /// ).await?;
    ///
    /// // Place order with human-readable values
    /// fixture.client.place_order(
    ///     "alice".to_string(),
    ///     fixture.market_id.clone(),
    ///     Side::Sell,
    ///     OrderType::Limit,
    ///     fixture.price_to_atoms(50000.0).to_string(), // $50,000 per BTC
    ///     fixture.to_base_atoms(1.0).to_string(),      // 1 BTC
    ///     "sig".to_string(),
    /// ).await?;
    /// ```
    ///
    /// # Example - Using custom decimals
    /// ```rust,ignore
    /// // Create exchange with realistic decimals: BTC=8, USDC=6
    /// let fixture = TestExchange::with_market_and_decimals("BTC", "USDC", 8, 6).await?;
    ///
    /// // Helper methods automatically use the correct decimals!
    /// let one_btc = fixture.to_base_atoms(1.0);      // 100_000_000 atoms (8 decimals)
    /// let fifty_k_usdc = fixture.to_quote_atoms(50000.0); // 50_000_000_000 atoms (6 decimals)
    /// ```
    pub async fn new() -> anyhow::Result<Self> {
        Self::with_market("BTC", "USDC").await
    }

    /// Create a test exchange with a custom market
    pub async fn with_market(base: &str, quote: &str) -> anyhow::Result<Self> {
        Self::with_market_and_decimals(base, quote, 6, 6).await
    }

    /// Create a test exchange with custom market and custom decimals
    pub async fn with_market_and_decimals(
        base: &str,
        quote: &str,
        base_decimals: u32,
        quote_decimals: u32,
    ) -> anyhow::Result<Self> {
        let server = TestServer::start().await?;
        let client = ExchangeClient::new(&server.base_url);

        // Setup tokens via admin API
        client
            .admin_create_token(base.to_string(), base_decimals as u8, format!("{} Token", base))
            .await?;
        client
            .admin_create_token(quote.to_string(), quote_decimals as u8, format!("{} Token", quote))
            .await?;

        // Setup market via admin API
        let market = client
            .admin_create_market(
                base.to_string(),
                quote.to_string(),
                1000,    // tick_size
                1000000, // lot_size
                1000000, // min_size
                10,      // maker_fee_bps (0.1%)
                20,      // taker_fee_bps (0.2%)
            )
            .await?;

        Ok(Self {
            server,
            client,
            market_id: market.id,
            base_ticker: base.to_string(),
            quote_ticker: quote.to_string(),
            base_decimals,
            quote_decimals,
        })
    }

    /// Create a user with starting balance (in atoms)
    ///
    /// Uses the admin faucet API to give users tokens, which also creates them if needed.
    pub async fn create_user_with_balance(
        &self,
        address: &str,
        base_amount: u128,
        quote_amount: u128,
    ) -> anyhow::Result<()> {
        // Use admin faucet to give tokens (this also creates user if needed)
        if base_amount > 0 {
            self.client
                .admin_faucet(
                    address.to_string(),
                    self.base_ticker.clone(),
                    base_amount.to_string(),
                )
                .await?;
        }
        if quote_amount > 0 {
            self.client
                .admin_faucet(
                    address.to_string(),
                    self.quote_ticker.clone(),
                    quote_amount.to_string(),
                )
                .await?;
        }

        Ok(())
    }

    /// Convert human-readable base token amount to atoms
    ///
    /// Example: `to_base_atoms(10.5)` with 6 decimals = 10_500_000 atoms
    ///
    /// # Usage in tests
    /// ```rust,ignore
    /// let fixture = TestExchange::with_market_and_decimals("BTC", "USDC", 8, 6).await?;
    ///
    /// // Instead of hardcoding: 100_000_000 atoms
    /// let btc_atoms = fixture.to_base_atoms(1.0); // Works with any decimal configuration!
    ///
    /// // Create user with 10 BTC
    /// fixture.create_user_with_balance("alice", fixture.to_base_atoms(10.0), 0).await?;
    /// ```
    pub fn to_base_atoms(&self, amount: f64) -> u128 {
        (amount * 10f64.powi(self.base_decimals as i32)) as u128
    }

    /// Convert human-readable quote token amount to atoms
    ///
    /// Example: `to_quote_atoms(50000.0)` with 6 decimals = 50_000_000_000 atoms
    pub fn to_quote_atoms(&self, amount: f64) -> u128 {
        (amount * 10f64.powi(self.quote_decimals as i32)) as u128
    }

    /// Convert base token atoms to human-readable amount
    ///
    /// Example: `from_base_atoms(10_500_000)` with 6 decimals = 10.5
    pub fn from_base_atoms(&self, atoms: u128) -> f64 {
        atoms as f64 / 10f64.powi(self.base_decimals as i32)
    }

    /// Convert quote token atoms to human-readable amount
    ///
    /// Example: `from_quote_atoms(50_000_000_000)` with 6 decimals = 50000.0
    pub fn from_quote_atoms(&self, atoms: u128) -> f64 {
        atoms as f64 / 10f64.powi(self.quote_decimals as i32)
    }

    /// Calculate price in atoms (quote atoms per whole base token)
    ///
    /// Example: `price_to_atoms(50000.0)` means 50000 USDC per 1 BTC
    /// With base_decimals=6: returns 50_000_000_000
    pub fn price_to_atoms(&self, price: f64) -> u128 {
        (price * 10f64.powi(self.quote_decimals as i32)) as u128
    }
}

/// Helper to wait for a condition with timeout
pub async fn wait_for<F, Fut>(mut condition: F, timeout_ms: u64) -> anyhow::Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    while start.elapsed().as_millis() < timeout_ms as u128 {
        if condition().await {
            return Ok(());
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    anyhow::bail!("Timeout waiting for condition")
}

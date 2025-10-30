use serde::{Deserialize, Serialize};

/// Bots configuration (from apps/bots/config.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub exchange: ExchangeConfig,
    pub accounts: AccountsConfig,
    pub funding: FundingConfig,
    pub orderbook_mirror: OrderbookMirrorConfig,
    pub trade_mirror: TradeMirrorConfig,
    pub hyperliquid: HyperliquidConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsConfig {
    pub maker_address: String,
    pub taker_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingConfig {
    pub btc_amount: String,
    pub usdc_amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookMirrorConfig {
    pub enabled: bool,
    pub coin: String,
    pub market_id: String,
    pub depth_levels: usize,
    pub update_interval_ms: u64,
    pub size_multiplier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeMirrorConfig {
    pub enabled: bool,
    pub coin: String,
    pub market_id: String,
    pub size_multiplier: String,
    pub min_trade_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperliquidConfig {
    pub ws_url: String,
}

impl Config {
    /// Load bots configuration from config.toml
    /// Uses CARGO_MANIFEST_DIR so the path is consistent regardless of where the binary is run from
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_path = std::env::var("BOTS_CONFIG")
            .unwrap_or_else(|_| format!("{}/config.toml", env!("CARGO_MANIFEST_DIR")));

        let builder = config::Config::builder()
            .add_source(config::File::with_name(&config_path).required(true));

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}

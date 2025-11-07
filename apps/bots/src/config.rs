use serde::{Deserialize, Serialize};

/// Bots configuration (from apps/bots/config.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub exchange: ExchangeConfig,
    pub markets: MarketsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsConfig {
    #[serde(default)]
    pub btc_usdc: Option<BtcUsdcMarketConfig>,
    #[serde(default)]
    pub bp_usdc: Option<BpUsdcMarketConfig>,
}

// ===========================
// BTC/USDC Market Configuration
// ===========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcUsdcMarketConfig {
    pub enabled: bool,
    #[serde(default)]
    pub orderbook_mirror: Option<BtcOrderbookMirrorConfig>,
    #[serde(default)]
    pub trade_mirror: Option<BtcTradeMirrorConfig>,
    pub hyperliquid: HyperliquidConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcOrderbookMirrorConfig {
    pub enabled: bool,
    pub user_address: String,
    pub depth_levels: usize,
    pub update_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcTradeMirrorConfig {
    pub enabled: bool,
    pub user_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperliquidConfig {
    pub ws_url: String,
}

// ===========================
// BP/USDC Market Configuration
// ===========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpUsdcMarketConfig {
    pub enabled: bool,
    #[serde(default)]
    pub lmsr: Option<LmsrConfig>,
    #[serde(default)]
    pub synthetic_trader: Option<SyntheticTraderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmsrConfig {
    pub enabled: bool,
    pub user_address: String,
    pub liquidity_param: f64, // b parameter in LMSR (controls market depth)
    pub initial_probability: f64, // Starting probability (0.0 - 1.0)
    pub update_interval_ms: u64, // How often to update quotes
    pub spread_bps: u64,      // Spread in basis points
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticTraderConfig {
    pub enabled: bool,
    pub user_address: String,
    pub min_interval_ms: u64, // Min time between trades
    pub max_interval_ms: u64, // Max time between trades
    pub min_size: f64,        // Min trade size
    pub max_size: f64,        // Max trade size
    pub buy_probability: f64, // Probability of buy vs sell (0.0-1.0)
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

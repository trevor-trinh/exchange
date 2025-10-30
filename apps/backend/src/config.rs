use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Backend configuration (from apps/backend/config.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub markets: Vec<MarketConfig>,
    pub tokens: Vec<TokenConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    pub base_ticker: String,
    pub quote_ticker: String,
    pub tick_size: String,
    pub lot_size: String,
    pub min_size: String,
    pub maker_fee_bps: i32,
    pub taker_fee_bps: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub ticker: String,
    pub decimals: u8,
    pub name: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try local config first (when running from apps/backend)
        let config_path = if std::path::Path::new("config.toml").exists() {
            "config.toml"
        } else if std::path::Path::new("apps/backend/config.toml").exists() {
            // When running from project root
            "apps/backend/config.toml"
        } else {
            anyhow::bail!("Could not find config.toml")
        };

        let contents = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

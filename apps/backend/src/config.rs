use serde::{Deserialize, Serialize};

/// Backend configuration (from apps/backend/config.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub markets: Vec<MarketConfig>,
    pub tokens: Vec<TokenConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
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
    /// Load backend configuration from config.toml
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let contents = std::fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Get server address
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get exchange URL for clients
    pub fn exchange_url(&self) -> String {
        format!("http://{}:{}", self.server.host, self.server.port)
    }
}

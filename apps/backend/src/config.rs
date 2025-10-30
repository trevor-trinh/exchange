use serde::{Deserialize, Serialize};
use std::path::Path;

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
    /// Load backend configuration from apps/backend/config.toml
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_path = std::env::var("BACKEND_CONFIG")
            .unwrap_or_else(|_| "apps/backend/config.toml".to_string());

        let builder = config::Config::builder()
            .add_source(config::File::with_name(&config_path).required(true))
            .add_source(
                config::Environment::with_prefix("BACKEND")
                    .separator("_")
                    .try_parsing(true),
            );

        let settings = builder.build()?;
        settings.try_deserialize()
    }

    /// Load from a specific path
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            .add_source(config::File::from(path.as_ref()))
            .add_source(
                config::Environment::with_prefix("EXCHANGE")
                    .separator("_")
                    .try_parsing(true),
            );

        let settings = builder.build()?;
        settings.try_deserialize()
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

use anyhow::{Context, Result};
use backend::config::Config;
use backend::db::Db;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env files for database URLs
    let _ = dotenvy::from_path(".env.defaults");
    let _ = dotenvy::from_path(".env");

    env_logger::init();

    println!("🚀 Initializing Exchange...\n");

    // Load backend configuration
    let config = Config::load().context("Failed to load config.toml")?;

    // Connect to database
    let db = Db::connect()
        .await
        .context("Failed to connect to database")?;
    log::info!("✅ Connected to databases");

    // Create tokens
    println!("📦 Creating tokens...");
    for token_config in &config.tokens {
        match db
            .create_token(
                token_config.ticker.clone(),
                token_config.decimals,
                token_config.name.clone(),
            )
            .await
        {
            Ok(_) => println!(
                "  ✓ Created token: {} ({}, {} decimals)",
                token_config.ticker, token_config.name, token_config.decimals
            ),
            Err(e) => {
                // Token might already exist, that's ok
                log::debug!(
                    "Token {} already exists or error: {}",
                    token_config.ticker,
                    e
                );
                println!("  ⊙ Token {} already exists", token_config.ticker);
            }
        }
    }

    // Create markets
    println!("\n🏪 Creating markets...");
    for market_config in &config.markets {
        let market_id = format!(
            "{}/{}",
            market_config.base_ticker, market_config.quote_ticker
        );

        // Parse string values to u128
        let tick_size = market_config
            .tick_size
            .parse::<u128>()
            .context("Invalid tick_size")?;
        let lot_size = market_config
            .lot_size
            .parse::<u128>()
            .context("Invalid lot_size")?;
        let min_size = market_config
            .min_size
            .parse::<u128>()
            .context("Invalid min_size")?;

        match db
            .create_market(
                market_config.base_ticker.clone(),
                market_config.quote_ticker.clone(),
                tick_size,
                lot_size,
                min_size,
                market_config.maker_fee_bps,
                market_config.taker_fee_bps,
            )
            .await
        {
            Ok(_) => println!(
                "  ✓ Created market: {} (tick: {}, lot: {}, min: {})",
                market_id, tick_size, lot_size, min_size
            ),
            Err(e) => {
                log::debug!("Market {} already exists or error: {}", market_id, e);
                println!("  ⊙ Market {} already exists", market_id);
            }
        }
    }

    println!("\n✨ Backend initialization complete!");
    println!("🚀 Start the backend: just backend");
    println!("🤖 Start the bots: just bots (bots will fund themselves)");

    Ok(())
}

use exchange_bots::hyperliquid::{HyperliquidClient, HlMessage};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("🔌 Fetching Hyperliquid BTC PERP orderbook with nSigFigs=5...\n");

    let client = HyperliquidClient::new("BTC".to_string());
    let (mut rx, _handle) = client.start().await?;

    let mut count = 0;
    while let Some(msg) = rx.recv().await {
        if let HlMessage::L2Book(book) = msg {
            if book.levels.len() >= 2 {
                let bids = &book.levels[0];
                let asks = &book.levels[1];

                println!("📊 BIDS: {} levels", bids.len());
                for (i, bid) in bids.iter().take(15).enumerate() {
                    println!("  #{:2}: price={:>10} size={:>10} orders={}",
                        i+1, bid.px, bid.sz, bid.n);
                }

                println!("\n📊 ASKS: {} levels", asks.len());
                for (i, ask) in asks.iter().take(15).enumerate() {
                    println!("  #{:2}: price={:>10} size={:>10} orders={}",
                        i+1, ask.px, ask.sz, ask.n);
                }

                count += 1;
                if count >= 2 {
                    println!("\n✅ Captured 2 snapshots");
                    break;
                }
                println!("\n{}\n", "─".repeat(60));
            }
        }
    }

    Ok(())
}

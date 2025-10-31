use exchange_sdk::ExchangeClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let exchange_url = std::env::var("EXCHANGE_URL").unwrap_or_else(|_| "http://localhost:8888".to_string());
    let client = ExchangeClient::new(&exchange_url);

    println!("🔍 Checking orders on exchange at {}\n", exchange_url);

    // Get orders for maker_bot (the main orderbook mirror bot)
    let maker_orders = client.get_orders("maker_bot", Some("BTC/USDC".to_string())).await?;
    let taker_orders = client.get_orders("taker_bot", Some("BTC/USDC".to_string())).await?;

    let mut all_orders = maker_orders;
    all_orders.extend(taker_orders);

    let open_orders: Vec<_> = all_orders.iter()
        .filter(|o| o.status.to_string() == "open")
        .collect();

    println!("📊 Total open orders: {}\n", open_orders.len());

    // Group by side
    let mut bids = 0;
    let mut asks = 0;
    let mut users = std::collections::HashSet::new();

    for order in &open_orders {
        users.insert(order.user_address.clone());
        match order.side.to_string().as_str() {
            "buy" => bids += 1,
            "sell" => asks += 1,
            _ => {}
        }
    }

    println!("📈 BUY orders: {}", bids);
    println!("📉 SELL orders: {}", asks);
    println!("\n👥 Users with open orders:");
    for user in &users {
        let user_orders = open_orders.iter().filter(|o| o.user_address == *user).count();
        println!("   - {}: {} orders", user, user_orders);
    }

    println!("\n💵 Sample orders:");
    println!("\nBUY side (bids) - top 5:");
    for (i, order) in open_orders.iter()
        .filter(|o| o.side.to_string() == "buy")
        .take(5)
        .enumerate()
    {
        println!("   #{}  Price: {:>10} Size: {:>10} User: {}",
            i+1, order.price, order.size, order.user_address);
    }

    println!("\nSELL side (asks) - top 5:");
    for (i, order) in open_orders.iter()
        .filter(|o| o.side.to_string() == "sell")
        .take(5)
        .enumerate()
    {
        println!("   #{}  Price: {:>10} Size: {:>10} User: {}",
            i+1, order.price, order.size, order.user_address);
    }

    Ok(())
}

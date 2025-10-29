use backend::engine::matcher::Matcher;
use backend::engine::orderbook::Orderbook;
use backend::models::domain::{Market, Order, OrderStatus, OrderType, Side};
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use uuid::Uuid;

fn create_test_market() -> Market {
    Market {
        id: "BTC/USDC".to_string(),
        base_ticker: "BTC".to_string(),
        quote_ticker: "USDC".to_string(),
        tick_size: 1000,
        lot_size: 1000000,
        min_size: 1000000,
        maker_fee_bps: 10,
        taker_fee_bps: 20,
    }
}

fn create_order(
    user: &str,
    market_id: &str,
    side: Side,
    price: u128,
    size: u128,
) -> Order {
    Order {
        id: Uuid::new_v4(),
        user_address: user.to_string(),
        market_id: market_id.to_string(),
        price,
        size,
        side,
        order_type: OrderType::Limit,
        status: OrderStatus::Pending,
        filled_size: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Benchmark single order matching against empty orderbook
fn bench_match_single_order_empty_book(c: &mut Criterion) {
    let market = create_test_market();

    c.bench_function("match_single_order_empty_book", |b| {
        b.iter(|| {
            let mut orderbook = Orderbook::new("BTC/USDC".to_string());
            let order = create_order("buyer", "BTC/USDC", Side::Buy, 50_000_000_000, 1_000_000);

            let matches = Matcher::match_order(black_box(&order), black_box(&mut orderbook));
            black_box(matches);
        });
    });
}

/// Benchmark matching against orderbook with various sizes
fn bench_match_order_against_book_sizes(c: &mut Criterion) {
    let market = create_test_market();
    let mut group = c.benchmark_group("match_order_book_size");

    for book_size in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*book_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(book_size),
            book_size,
            |b, &size| {
                b.iter(|| {
                    let mut orderbook = Orderbook::new("BTC/USDC".to_string());

                    // Fill orderbook with sell orders
                    for i in 0..size {
                        let sell_order = create_order(
                            &format!("seller{}", i),
                            "BTC/USDC",
                            Side::Sell,
                            50_000_000_000 + (i as u128 * 1000),
                            1_000_000,
                        );
                        orderbook.add_order(sell_order);
                    }

                    // Match a large buy order
                    let buy_order = create_order(
                        "buyer",
                        "BTC/USDC",
                        Side::Buy,
                        60_000_000_000,
                        (size as u128) * 1_000_000,
                    );

                    let matches = Matcher::match_order(black_box(&buy_order), black_box(&mut orderbook));
                    black_box(matches);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark partial fills
fn bench_partial_fill_matching(c: &mut Criterion) {
    let market = create_test_market();

    c.bench_function("partial_fill_matching", |b| {
        b.iter(|| {
            let mut orderbook = Orderbook::new("BTC/USDC".to_string());

            // Add 10 sell orders at different prices
            for i in 0..10 {
                let sell_order = create_order(
                    &format!("seller{}", i),
                    "BTC/USDC",
                    Side::Sell,
                    50_000_000_000 + (i as u128 * 10_000),
                    1_000_000,
                );
                orderbook.add_order(sell_order);
            }

            // Match with smaller buy order (only fills 3)
            let buy_order = create_order(
                "buyer",
                "BTC/USDC",
                Side::Buy,
                50_050_000_000,
                3_000_000,
            );

            let matches = Matcher::match_order(black_box(&buy_order), black_box(&mut orderbook));
            black_box(matches);
        });
    });
}

/// Benchmark market order execution
fn bench_market_order_execution(c: &mut Criterion) {
    let market = create_test_market();

    c.bench_function("market_order_execution", |b| {
        b.iter(|| {
            let mut orderbook = Orderbook::new("BTC/USDC".to_string());

            // Add orderbook depth
            for i in 0..100 {
                let sell_order = create_order(
                    &format!("seller{}", i),
                    "BTC/USDC",
                    Side::Sell,
                    50_000_000_000 + (i as u128 * 1000),
                    1_000_000,
                );
                orderbook.add_order(sell_order);
            }

            // Execute market order
            let market_order = Order {
                id: Uuid::new_v4(),
                user_address: "buyer".to_string(),
                market_id: "BTC/USDC".to_string(),
                price: 0,
                size: 10_000_000,
                side: Side::Buy,
                order_type: OrderType::Market,
                status: OrderStatus::Pending,
                filled_size: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let matches = Matcher::match_order(black_box(&market_order), black_box(&mut orderbook));
            black_box(matches);
        });
    });
}

/// Benchmark price-time priority sorting
fn bench_price_time_priority(c: &mut Criterion) {
    let market = create_test_market();

    c.bench_function("price_time_priority", |b| {
        b.iter(|| {
            let mut orderbook = Orderbook::new("BTC/USDC".to_string());

            // Add 50 orders at same price (FIFO should apply)
            let price = 50_000_000_000;
            for i in 0..50 {
                let sell_order = create_order(
                    &format!("seller{}", i),
                    "BTC/USDC",
                    Side::Sell,
                    price,
                    1_000_000,
                );
                orderbook.add_order(sell_order);
                // Small delay to ensure different timestamps
                std::thread::sleep(std::time::Duration::from_nanos(1));
            }

            // Match buy order
            let buy_order = create_order(
                "buyer",
                "BTC/USDC",
                Side::Buy,
                price,
                10_000_000,
            );

            let matches = Matcher::match_order(black_box(&buy_order), black_box(&mut orderbook));
            black_box(matches);
        });
    });
}

criterion_group!(
    benches,
    bench_match_single_order_empty_book,
    bench_match_order_against_book_sizes,
    bench_partial_fill_matching,
    bench_market_order_execution,
    bench_price_time_priority,
);
criterion_main!(benches);

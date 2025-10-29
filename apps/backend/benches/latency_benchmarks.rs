use backend::engine::matcher::Matcher;
use backend::engine::orderbook::Orderbook;
use backend::models::domain::{Market, Order, OrderStatus, OrderType, Side};
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
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

/// Benchmark order matching latency with realistic orderbook
fn bench_order_matching_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_matching_latency");
    group.sample_size(1000);
    group.measurement_time(Duration::from_secs(10));

    // Pre-populate orderbook with realistic depth
    let mut orderbook = Orderbook::new("BTC/USDC".to_string());
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

    group.bench_function("match_taker_buy", |b| {
        b.iter(|| {
            let buy_order = create_order(
                "buyer",
                "BTC/USDC",
                Side::Buy,
                50_010_000_000,
                5_000_000,
            );
            let matches = Matcher::match_order(black_box(&buy_order), black_box(&mut orderbook));
            black_box(matches);
        });
    });

    group.finish();
}

/// Benchmark critical path latency (place + match + fill)
fn bench_critical_path_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("critical_path_latency");
    group.sample_size(500);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("place_match_fill_cycle", |b| {
        b.iter(|| {
            let mut orderbook = Orderbook::new("BTC/USDC".to_string());

            // Place maker order
            let maker = create_order(
                "seller",
                "BTC/USDC",
                Side::Sell,
                50_000_000_000,
                10_000_000,
            );
            orderbook.add_order(maker);

            // Place taker order that matches
            let taker = create_order(
                "buyer",
                "BTC/USDC",
                Side::Buy,
                50_000_000_000,
                10_000_000,
            );

            let matches = Matcher::match_order(black_box(&taker), black_box(&mut orderbook));
            black_box(matches);
        });
    });

    group.finish();
}

/// Benchmark orderbook add operation latency
fn bench_orderbook_add_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook_add_latency");
    group.sample_size(1000);

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut orderbook = Orderbook::new("BTC/USDC".to_string());
                    // Pre-populate
                    for i in 0..size {
                        let order = create_order(
                            &format!("seller{}", i),
                            "BTC/USDC",
                            Side::Sell,
                            50_000_000_000 + (i as u128 * 1000),
                            1_000_000,
                        );
                        orderbook.add_order(order);
                    }
                    orderbook
                },
                |mut orderbook| {
                    let new_order = create_order(
                        "new_seller",
                        "BTC/USDC",
                        Side::Sell,
                        50_500_000_000,
                        1_000_000,
                    );
                    orderbook.add_order(black_box(new_order));
                    black_box(orderbook);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark order cancellation latency
fn bench_cancel_order_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cancel_order_latency");
    group.sample_size(1000);

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut orderbook = Orderbook::new("BTC/USDC".to_string());
                    let mut order_ids = Vec::new();

                    // Pre-populate
                    for i in 0..size {
                        let order = create_order(
                            &format!("seller{}", i),
                            "BTC/USDC",
                            Side::Sell,
                            50_000_000_000 + (i as u128 * 1000),
                            1_000_000,
                        );
                        order_ids.push(order.id);
                        orderbook.add_order(order);
                    }
                    (orderbook, order_ids)
                },
                |(mut orderbook, order_ids)| {
                    // Remove middle order
                    let cancel_id = order_ids[size / 2];
                    let result = orderbook.remove_order(black_box(cancel_id));
                    black_box(result);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark orderbook snapshot generation latency
fn bench_snapshot_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_generation_latency");
    group.sample_size(500);

    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut orderbook = Orderbook::new("BTC/USDC".to_string());

            // Add both bids and asks
            for i in 0..size / 2 {
                let sell_order = create_order(
                    &format!("seller{}", i),
                    "BTC/USDC",
                    Side::Sell,
                    50_000_000_000 + (i as u128 * 1000),
                    1_000_000,
                );
                orderbook.add_order(sell_order);

                let buy_order = create_order(
                    &format!("buyer{}", i),
                    "BTC/USDC",
                    Side::Buy,
                    49_000_000_000 - (i as u128 * 1000),
                    1_000_000,
                );
                orderbook.add_order(buy_order);
            }

            b.iter(|| {
                let snapshot = orderbook.snapshot();
                black_box(snapshot);
            });
        });
    }

    group.finish();
}

/// Benchmark worst-case latency (market order sweeping deep book)
fn bench_worst_case_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("worst_case_latency");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("market_sweep_1000_levels", |b| {
        b.iter_batched(
            || {
                let mut orderbook = Orderbook::new("BTC/USDC".to_string());
                // Fill with 1000 price levels
                for i in 0..1000 {
                    let sell_order = create_order(
                        &format!("seller{}", i),
                        "BTC/USDC",
                        Side::Sell,
                        50_000_000_000 + (i as u128 * 1000),
                        1_000_000,
                    );
                    orderbook.add_order(sell_order);
                }
                orderbook
            },
            |mut orderbook| {
                // Large market order sweeping through 500 levels
                let market_order = Order {
                    id: Uuid::new_v4(),
                    user_address: "buyer".to_string(),
                    market_id: "BTC/USDC".to_string(),
                    price: 0,
                    size: 500_000_000, // 500 BTC
                    side: Side::Buy,
                    order_type: OrderType::Market,
                    status: OrderStatus::Pending,
                    filled_size: 0,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                let matches = Matcher::match_order(black_box(&market_order), black_box(&mut orderbook));
                black_box(matches);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark best-case latency (immediate match at best price)
fn bench_best_case_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("best_case_latency");
    group.sample_size(1000);

    group.bench_function("immediate_match_top_of_book", |b| {
        b.iter_batched(
            || {
                let mut orderbook = Orderbook::new("BTC/USDC".to_string());
                let sell_order = create_order(
                    "seller",
                    "BTC/USDC",
                    Side::Sell,
                    50_000_000_000,
                    10_000_000,
                );
                orderbook.add_order(sell_order);
                orderbook
            },
            |mut orderbook| {
                let buy_order = create_order(
                    "buyer",
                    "BTC/USDC",
                    Side::Buy,
                    50_000_000_000,
                    5_000_000,
                );
                let matches = Matcher::match_order(black_box(&buy_order), black_box(&mut orderbook));
                black_box(matches);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_order_matching_latency,
    bench_critical_path_latency,
    bench_orderbook_add_latency,
    bench_cancel_order_latency,
    bench_snapshot_latency,
    bench_worst_case_latency,
    bench_best_case_latency,
);
criterion_main!(benches);

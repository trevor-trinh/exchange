use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Benchmark fee calculation
fn bench_fee_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("fee_calculation");

    for fee_bps in [0, 10, 20, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(fee_bps),
            fee_bps,
            |b, &fee_bps| {
                b.iter(|| {
                    let trade_size = 1_000_000_u128;
                    let fee = black_box((trade_size as i128 * fee_bps as i128 / 10000) as u128);
                    black_box(fee);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark fee calculation on various trade sizes
fn bench_fee_calculation_trade_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("fee_calculation_trade_sizes");
    let fee_bps = 20; // 0.2%

    for trade_size in [1_000_000, 10_000_000, 100_000_000, 1_000_000_000].iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(trade_size),
            trade_size,
            |b, &size| {
                b.iter(|| {
                    let fee = black_box((size as i128 * fee_bps / 10000) as u128);
                    black_box(fee);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark balance calculation (locked + available)
fn bench_balance_available_calculation(c: &mut Criterion) {
    c.bench_function("balance_available_calculation", |b| {
        b.iter(|| {
            let total = 1_000_000_000_u128;
            let locked = 300_000_000_u128;
            let available = black_box(total - locked);
            black_box(available);
        });
    });
}

/// Benchmark bulk fee calculations (simulating many trades)
fn bench_bulk_fee_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_fee_calculations");

    for num_trades in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*num_trades as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_trades),
            num_trades,
            |b, &count| {
                b.iter(|| {
                    let mut total_fees = 0_u128;
                    for _ in 0..count {
                        let trade_size = 1_000_000_u128;
                        let fee = (trade_size as i128 * 20 / 10000) as u128;
                        total_fees += fee;
                    }
                    black_box(total_fees);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark overflow checking for balance operations
fn bench_balance_overflow_check(c: &mut Criterion) {
    c.bench_function("balance_overflow_check", |b| {
        b.iter(|| {
            let price = 50_000_000_000_u128;
            let size = 10_000_000_u128;

            // Checked multiplication (what we use in real code)
            let result = black_box(price.checked_mul(size));
            black_box(result);
        });
    });
}

/// Benchmark maker vs taker fee determination
fn bench_fee_role_determination(c: &mut Criterion) {
    use backend::models::domain::Side;

    c.bench_function("fee_role_determination", |b| {
        b.iter(|| {
            let taker_side = Side::Buy;
            let maker_fee_bps = 10;
            let taker_fee_bps = 20;

            let (buyer_fee_bps, seller_fee_bps) = match black_box(taker_side) {
                Side::Buy => (taker_fee_bps, maker_fee_bps),
                Side::Sell => (maker_fee_bps, taker_fee_bps),
            };

            black_box((buyer_fee_bps, seller_fee_bps));
        });
    });
}

criterion_group!(
    benches,
    bench_fee_calculation,
    bench_fee_calculation_trade_sizes,
    bench_balance_available_calculation,
    bench_bulk_fee_calculations,
    bench_balance_overflow_check,
    bench_fee_role_determination,
);
criterion_main!(benches);

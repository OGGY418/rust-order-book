use criterion::{black_box, criterion_group, criterion_main, Criterion};
use order_book_hybrid::engine::orderbook::OrderBook;
use order_book_hybrid::engine::order::OrderSide;
use std::time::{SystemTime, UNIX_EPOCH};

fn benchmark_add_order(c: &mut Criterion) {
    let orderbook = OrderBook::new();
    
    c.bench_function("add_order", |b| {
        b.iter(|| {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            
            orderbook.add_order(
                black_box(OrderSide::Bid),
                black_box(100.0),
                black_box(1.0),
                timestamp,
                "user1".to_string(),
            );
        });
    });
}

fn benchmark_match_orders(c: &mut Criterion) {
    c.bench_function("match_orders", |b| {
        b.iter(|| {
            let orderbook = OrderBook::new();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            
            
            orderbook.add_order(
                OrderSide::Ask,
                100.0,
                1.0,
                timestamp,
                "seller".to_string(),
            );
            
            
            orderbook.add_order(
                OrderSide::Bid,
                100.0,
                1.0,
                timestamp + 1,
                "buyer".to_string(),
            );
        });
    });
}

fn benchmark_get_depth(c: &mut Criterion) {
    let orderbook = OrderBook::new();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    
    for i in 0..100 {
        orderbook.add_order(
            OrderSide::Bid,
            100.0 - i as f64,
            1.0,
            timestamp,
            format!("user{}", i),
        );
        orderbook.add_order(
            OrderSide::Ask,
            101.0 + i as f64,
            1.0,
            timestamp,
            format!("user{}", i + 100),
        );
    }
    
    c.bench_function("get_depth", |b| {
        b.iter(|| {
            black_box(orderbook.get_market_depth(20));
        });
    });
}

fn benchmark_high_frequency(c: &mut Criterion) {
    c.bench_function("high_frequency_1000_orders", |b| {
        b.iter(|| {
            let orderbook = OrderBook::new();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            
            for i in 0..1000 {
                let side = if i % 2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
                let price = if side == OrderSide::Bid { 99.0 } else { 101.0 };
                
                orderbook.add_order(
                    side,
                    price,
                    1.0,
                    timestamp + i as u64,
                    format!("user{}", i),
                );
            }
        });
    });
}

criterion_group!(
    benches,
    benchmark_add_order,
    benchmark_match_orders,
    benchmark_get_depth,
    benchmark_high_frequency
);
criterion_main!(benches);
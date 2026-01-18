use std::sync::Arc;
use std::collections::HashMap;
use actix_web::{web::{self, Data}, App, HttpServer};
use actix_cors::Cors;
use order_book_hybrid::engine::orderbook::OrderBook;
use order_book_hybrid::api::{routes, websocket};
use order_book_hybrid::exchange::{BinanceWebSocket, CoinbaseWebSocket, BybitWebSocket, Coin};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    log::info!("═══════════════════════════════════════");
    log::info!(" Starting Order Book Hybrid Server...");
    log::info!("═══════════════════════════════════════");
    
   
    let btc_orderbook = Arc::new(OrderBook::new());
    let sol_orderbook = Arc::new(OrderBook::new());
    let eth_orderbook = Arc::new(OrderBook::new());

    log::info!("✅ Multi-coin OrderBooks initialized:");
    log::info!("   • Bitcoin (BTC)");
    log::info!("   • Solana (SOL)");
    log::info!("   • Ethereum (ETH)");
    log::info!("");
    
    log::info!("═══════════════════════════════");
    log::info!("Lock-free OrderBook initialized");
    log::info!("═══════════════════════════════");
    log::info!(" Using:");
    log::info!("   - parking_lot::RwLock for BTreeMap");
    log::info!("   - DashMap for order storage");
    log::info!("   - SegQueue for FIFO ordering");
    log::info!("   - AtomicU64 for counters");

    log::info!("═══════════════════════════════");
    log::info!(" Starting Multi-Exchange Real-Time Data Feeds...");
    log::info!("");
    log::info!("═══════════════════════════════");

    log::info!(" Starting Bitcoin (BTC) Feeds...");
    BinanceWebSocket::start(btc_orderbook.clone(), Coin::BTC);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    CoinbaseWebSocket::start(btc_orderbook.clone(), Coin::BTC);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    BybitWebSocket::start(btc_orderbook.clone(), Coin::BTC);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    log::info!(" BTC feeds: Binance + Coinbase + Bybit");
    log::info!("");

    log::info!(" Starting Solana (SOL) Feeds...");
    BinanceWebSocket::start(sol_orderbook.clone(), Coin::SOL);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    CoinbaseWebSocket::start(sol_orderbook.clone(), Coin::SOL);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    BybitWebSocket::start(sol_orderbook.clone(), Coin::SOL);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    log::info!(" SOL feeds: Binance + Coinbase + Bybit");
    log::info!("");

    log::info!(" Starting Ethereum (ETH) Feeds...");
    BinanceWebSocket::start(eth_orderbook.clone(), Coin::ETH);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    CoinbaseWebSocket::start(eth_orderbook.clone(), Coin::ETH);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    BybitWebSocket::start(eth_orderbook.clone(), Coin::ETH);
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    log::info!(" ETH feeds: Binance + Coinbase + Bybit");
    log::info!("");

      log::info!("═══════════════════════════════════════");
    log::info!(" All exchanges streaming live data!");
    log::info!("═══════════════════════════════════════");
    
    let orderbook = btc_orderbook.clone();
    
    log::info!("═══════════════════════════════════════");
    log::info!("  HTTP server on http://127.0.0.1:8080");
    log::info!("═══════════════════════════════════════");



    log::info!("═══════════════════════════════════════");
    log::info!("WebSocket available at ws://127.0.0.1:8080/ws");
     log::info!("═══════════════════════════════════════");
    log::info!("");
     log::info!(" Available endpoints:");
    log::info!("   GET  /health           - Health check");
    log::info!("   GET  /depth            - Order book depth");
    log::info!("   GET  /stats            - Statistics");
    log::info!("   POST /order            - Create order");
    log::info!("   DELETE /order          - Cancel order");
    log::info!("   GET  /ws               - WebSocket stream");
    log::info!("═══════════════════════════════════════");
    log::info!(" Server ready! Accepting connections...");
    log::info!("");
    
    HttpServer::new(move || {

        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(Data::new(orderbook.clone()))
            .service(routes::health_check)
            .service(routes::get_depth)
            .service(routes::create_order)
            .service(routes::delete_order)
            .service(routes::get_stats)
            .route("/ws", web::get().to(websocket::ws_index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
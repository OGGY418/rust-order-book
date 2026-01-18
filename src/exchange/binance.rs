use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::sync::Arc;
use url::Url;

use crate::engine::orderbook::OrderBook;
use crate::engine::order::OrderSide;

#[derive(Debug, Deserialize, Serialize)]
struct BinanceTrade {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    quantity: String,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}

#[derive(Debug, Clone)]
pub enum Coin {
    BTC,
    ETH,
    SOL,
}

impl Coin {
    pub fn symbol(&self) -> &str {
        match self {
            Coin::BTC => "btcusdt",
            Coin::ETH => "ethusdt",
            Coin::SOL => "solusdt",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Coin::BTC => "Bitcoin",
            Coin::ETH => "Ethereum",
            Coin::SOL => "Solana",
        }
    }
}

pub struct BinanceWebSocket {
    orderbook: Arc<OrderBook>,
    coin: Coin,
}

impl BinanceWebSocket {
    pub fn new(orderbook: Arc<OrderBook>, coin: Coin) -> Self {
        Self { orderbook, coin }
    }

    
    pub async fn connect(&self) -> Result<(), String> {
        let symbol = self.coin.symbol();
        let url = format!("wss://stream.binance.com:9443/ws/{}@trade", symbol);
        
        log::info!("🌐 Connecting to Binance WebSocket: {}", url);
        
        let url = Url::parse(&url).map_err(|e| e.to_string())?;
        let (ws_stream, _) = connect_async(url).await.map_err(|e| e.to_string())?;
        
        log::info!("✅ Connected to Binance for {}", self.coin.display_name());
        
        let (mut _write, mut read) = ws_stream.split();
        
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(trade) = serde_json::from_str::<BinanceTrade>(&text) {
                        self.process_trade(trade).await;
                    }
                }
                Ok(Message::Close(_)) => {
                    log::warn!(" Binance WebSocket closed");
                    break;
                }
                Err(e) => {
                    log::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    async fn process_trade(&self, trade: BinanceTrade) {
        let price: f64 = match trade.price.parse() {
            Ok(p) => p,
            Err(_) => return,
        };
        
        let quantity: f64 = match trade.quantity.parse() {
            Ok(q) => q,
            Err(_) => return,
        };
        
        
        
        let side = if trade.is_buyer_maker {
            OrderSide::Ask 
        } else {
            OrderSide::Bid 
        };
        
        
        self.add_market_depth(price, quantity, side);
        
        log::debug!(
            "📊 {} Trade: {} @ ${:.2} ({})",
            self.coin.display_name(),
            quantity,
            price,
            if trade.is_buyer_maker { "SELL" } else { "BUY" }
        );
    }

   
    fn add_market_depth(&self, current_price: f64, quantity: f64, _side: OrderSide) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        
        for i in 1..=5 {
            let bid_price = current_price - (i as f64 * 0.5);
            let bid_qty = quantity * (1.0 + (i as f64 * 0.1));
            
            self.orderbook.add_order(
                OrderSide::Bid,
                bid_price,
                bid_qty,
                timestamp,
                format!("binance_bid_{}", i),
            );
        }
        
        
        for i in 1..=5 {
            let ask_price = current_price + (i as f64 * 0.5);
            let ask_qty = quantity * (1.0 + (i as f64 * 0.1));
            
            self.orderbook.add_order(
                OrderSide::Ask,
                ask_price,
                ask_qty,
                timestamp,
                format!("binance_ask_{}", i),
            );
        }
    }

    
    pub fn start(orderbook: Arc<OrderBook>, coin: Coin) {
        tokio::spawn(async move {
            let ws = BinanceWebSocket::new(orderbook, coin);
            
            loop {
                if let Err(e) = ws.connect().await {
                    log::error!("Binance connection error: {}", e);
                    log::info!("🔄 Reconnecting in 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });
    }
}


pub struct MultiCoinBinance {
    orderbooks: Vec<(Coin, Arc<OrderBook>)>,
}

impl MultiCoinBinance {
    pub fn new() -> Self {
        Self {
            orderbooks: Vec::new(),
        }
    }

    pub fn add_coin(&mut self, coin: Coin, orderbook: Arc<OrderBook>) {
        self.orderbooks.push((coin, orderbook));
    }

   
    pub fn start_all(&self) {
        for (coin, orderbook) in &self.orderbooks {
            log::info!("Starting {} feed", coin.display_name());
            BinanceWebSocket::start(orderbook.clone(), coin.clone());
        }
    }
}
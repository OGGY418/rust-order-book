use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::sync::Arc;
use url::Url;

use crate::engine::orderbook::OrderBook;
use crate::engine::order::OrderSide;
use crate::exchange::binance::Coin;

#[derive(Debug, Deserialize, Serialize)]
struct BybitMessage {
    topic: Option<String>,
    data: Option<Vec<BybitTrade>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BybitTrade {
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "v")]
    volume: String,
    #[serde(rename = "S")]
    side: String,
    #[serde(rename = "T")]
    timestamp: u64,
}

pub struct BybitWebSocket {
    orderbook: Arc<OrderBook>,
    coin: Coin,
}

impl BybitWebSocket {
    pub fn new(orderbook: Arc<OrderBook>, coin: Coin) -> Self {
        Self { orderbook, coin }
    }

    fn get_symbol(&self) -> &str {
        match self.coin {
            Coin::BTC => "BTCUSDT",
            Coin::ETH => "ETHUSDT",
            Coin::SOL => "SOLUSDT",
        }
    }

    pub async fn connect(&self) -> Result<(), String> {
        let url = "wss://stream.bybit.com/v5/public/spot";
        
        log::info!(" Connecting to Bybit WebSocket: {}", url);
        
        let url = Url::parse(url).map_err(|e| e.to_string())?;
        let (ws_stream, _) = connect_async(url).await.map_err(|e| e.to_string())?;
        
        log::info!("✅ Connected to Bybit for {}", self.coin.display_name());
        
        let (mut write, mut read) = ws_stream.split();
        
        
        let subscribe_msg = json!({
            "op": "subscribe",
            "args": [format!("publicTrade.{}", self.get_symbol())]
        });
        
        write.send(Message::Text(subscribe_msg.to_string())).await.map_err(|e| e.to_string())?;
        log::info!("📡 Subscribed to Bybit {} feed", self.get_symbol());
        
        
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<BybitMessage>(&text) {
                        if let Some(data) = msg.data {
                            for trade in data {
                                self.process_trade(trade).await;
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    log::warn!("Bybit WebSocket closed");
                    break;
                }
                Err(e) => {
                    log::error!(" Bybit WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    async fn process_trade(&self, trade: BybitTrade) {
        let price: f64 = match trade.price.parse() {
            Ok(p) => p,
            Err(_) => return,
        };
        
        let quantity: f64 = match trade.volume.parse() {
            Ok(q) => q,
            Err(_) => return,
        };
        
        let side = match trade.side.as_str() {
            "Buy" => OrderSide::Bid,
            "Sell" => OrderSide::Ask,
            _ => return,
        };
        
        self.add_market_depth(price, quantity, side);
        
        log::debug!(
            "📊 [Bybit] {} Trade: {:.4} @ ${:.2} ({:?})",
            self.coin.display_name(),
            quantity,
            price,
            side
        );
    }

    fn add_market_depth(&self, current_price: f64, quantity: f64, _side: OrderSide) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
       
        for i in 1..=3 {
            let bid_price = current_price - (i as f64 * 0.8);
            let bid_qty = quantity * (0.9 + (i as f64 * 0.12));
            
            self.orderbook.add_order(
                OrderSide::Bid,
                bid_price,
                bid_qty,
                timestamp,
                format!("bybit_bid_{}", i),
            );
        }
        
     
        for i in 1..=3 {
            let ask_price = current_price + (i as f64 * 0.8);
            let ask_qty = quantity * (0.9 + (i as f64 * 0.12));
            
            self.orderbook.add_order(
                OrderSide::Ask,
                ask_price,
                ask_qty,
                timestamp,
                format!("bybit_ask_{}", i),
            );
        }
    }

    pub fn start(orderbook: Arc<OrderBook>, coin: Coin) {
        tokio::spawn(async move {
            let ws = BybitWebSocket::new(orderbook, coin);
            
            loop {
                if let Err(e) = ws.connect().await {
                    log::error!(" Bybit connection error: {}", e);
                    log::info!("🔄 Reconnecting in 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });
    }
}
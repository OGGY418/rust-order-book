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
struct CoinbaseMatch {
    #[serde(rename = "type")]
    msg_type: String,
    product_id: String,
    price: Option<String>,
    size: Option<String>,
    side: Option<String>,
    time: Option<String>,
}

pub struct CoinbaseWebSocket {
    orderbook: Arc<OrderBook>,
    coin: Coin,
}

impl CoinbaseWebSocket {
    pub fn new(orderbook: Arc<OrderBook>, coin: Coin) -> Self {
        Self { orderbook, coin }
    }

    fn get_product_id(&self) -> &str {
        match self.coin {
            Coin::BTC => "BTC-USD",
            Coin::ETH => "ETH-USD",
            Coin::SOL => "SOL-USD",
        }
    }

    pub async fn connect(&self) -> Result<(), String> {
        let url = "wss://ws-feed.exchange.coinbase.com";
        
        log::info!(" Connecting to Coinbase WebSocket: {}", url);
        
        let url = Url::parse(url).map_err(|e| e.to_string())?;
        let (ws_stream, _) = connect_async(url).await.map_err(|e| e.to_string())?;
        
        log::info!("✅ Connected to Coinbase for {}", self.coin.display_name());
        
        let (mut write, mut read) = ws_stream.split();
        
        
        let subscribe_msg = json!({
            "type": "subscribe",
            "product_ids": [self.get_product_id()],
            "channels": ["matches"]
        });
        
        write.send(Message::Text(subscribe_msg.to_string())).await.map_err(|e| e.to_string())?;
        log::info!("📡 Subscribed to Coinbase {} feed", self.get_product_id());
        
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(trade) = serde_json::from_str::<CoinbaseMatch>(&text) {
                        if trade.msg_type == "match" {
                            self.process_trade(trade).await;
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    log::warn!("Coinbase WebSocket closed");
                    break;
                }
                Err(e) => {
                    log::error!(" Coinbase WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    async fn process_trade(&self, trade: CoinbaseMatch) {
        let price: f64 = match trade.price.and_then(|p| p.parse().ok()) {
            Some(p) => p,
            None => return,
        };
        
        let quantity: f64 = match trade.size.and_then(|q| q.parse().ok()) {
            Some(q) => q,
            None => return,
        };
        
        let side = match trade.side.as_deref() {
            Some("buy") => OrderSide::Bid,
            Some("sell") => OrderSide::Ask,
            _ => return,
        };
        
        self.add_market_depth(price, quantity, side);
        
        log::debug!(
            "📊 [Coinbase] {} Trade: {:.4} @ ${:.2} ({:?})",
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
            let bid_price = current_price - (i as f64 * 1.0);
            let bid_qty = quantity * (0.8 + (i as f64 * 0.15));
            
            self.orderbook.add_order(
                OrderSide::Bid,
                bid_price,
                bid_qty,
                timestamp,
                format!("coinbase_bid_{}", i),
            );
        }
        
      
        for i in 1..=3 {
            let ask_price = current_price + (i as f64 * 1.0);
            let ask_qty = quantity * (0.8 + (i as f64 * 0.15));
            
            self.orderbook.add_order(
                OrderSide::Ask,
                ask_price,
                ask_qty,
                timestamp,
                format!("coinbase_ask_{}", i),
            );
        }
    }

    pub fn start(orderbook: Arc<OrderBook>, coin: Coin) {
        tokio::spawn(async move {
            let ws = CoinbaseWebSocket::new(orderbook, coin);
            
            loop {
                if let Err(e) = ws.connect().await {
                    log::error!(" Coinbase connection error: {}", e);
                    log::info!("🔄 Reconnecting in 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });
    }
}
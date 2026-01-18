use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::engine::orderbook::OrderBook;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
   
    DepthUpdate {
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    },
  
    TradeExecuted {
        price: f64,
        quantity: f64,
        side: String,
        timestamp: u64,
    },
    
    OrderUpdate {
        order_id: String,
        status: String,
        filled_quantity: f64,
    },
    
    StatsUpdate {
        best_bid: Option<f64>,
        best_ask: Option<f64>,
        spread: Option<f64>,
        volume_24h: f64,
    },

    Pong,
}


pub struct OrderBookWebSocket {
    
    hb: Instant,
    
    orderbook: Arc<OrderBook>,
}

impl OrderBookWebSocket {
    pub fn new(orderbook: Arc<OrderBook>) -> Self {
        Self {
            hb: Instant::now(),
            orderbook,
        }
    }

    
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("WebSocket client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }

    
    fn send_depth(&self, ctx: &mut ws::WebsocketContext<Self>) {
        let (bids, asks) = self.orderbook.get_market_depth(20);
        
        let msg = WsMessage::DepthUpdate { bids, asks };
        
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }

    
    fn send_stats(&self, ctx: &mut ws::WebsocketContext<Self>) {
        let stats = self.orderbook.get_stats();
        
        let msg = WsMessage::StatsUpdate {
            best_bid: stats.best_bid,
            best_ask: stats.best_ask,
            spread: stats.spread,
            volume_24h: stats.total_volume_traded,
        };
        
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }
}

impl Actor for OrderBookWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebSocket connection established");
        self.hb(ctx);
        
        
        self.send_depth(ctx);
        self.send_stats(ctx);
        
        
        ctx.run_interval(Duration::from_millis(100), |act, ctx| {
            act.send_depth(ctx);
        });
        
        
        ctx.run_interval(Duration::from_secs(1), |act, ctx| {
            act.send_stats(ctx);
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        println!("WebSocket connection closed");
    }
}


impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for OrderBookWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // Handle client commands
                match text.trim() {
                    "depth" => self.send_depth(ctx),
                    "stats" => self.send_stats(ctx),
                    _ => {
                        println!("Unknown command: {}", text);
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                println!("Unexpected binary message");
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}


pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    orderbook: web::Data<Arc<OrderBook>>,
) -> Result<HttpResponse, Error> {
    let ws = OrderBookWebSocket::new(orderbook.get_ref().clone());
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
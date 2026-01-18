use serde::{Deserialize, Serialize};
use crate::engine::trade::Trade;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub price: f64,
    pub quantity: f64,
    pub user_id: String,
    pub side: Side,
    #[serde(default = "default_order_type")]
    pub order_type: OrderType,
}

fn default_order_type() -> OrderType {
    OrderType::Limit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub order_id: String,
    pub filled_quantity: f64,
    pub remaining_quantity: f64,
    pub average_price: f64,
    pub fills: Vec<Fill>,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub trade_id: String,
    pub quantity: f64,
    pub price: f64,
    pub maker_order_id: String,
    pub taker_order_id: String,
    pub timestamp: u64,
}

impl From<&Trade> for Fill {
    fn from(trade: &Trade) -> Self {
        Self {
            trade_id: format!("{}_{}", trade.bid_order_id, trade.ask_order_id),
            quantity: trade.quantity,
            price: trade.price,
            maker_order_id: trade.bid_order_id.to_string(),
            taker_order_id: trade.ask_order_id.to_string(),
            timestamp: trade.timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteOrderRequest {
    pub order_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteOrderResponse {
    pub success: bool,
    pub remaining_quantity: f64,
    pub filled_quantity: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepthResponse {
    pub bids: Vec<DepthLevel>,
    pub asks: Vec<DepthLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthLevel {
    pub price: f64,
    pub quantity: f64,
}
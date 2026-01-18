use crate::engine::price::Price;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub side: OrderSide,
    pub price: Price,
    pub quantity: f64,
    pub timestamp: u64,
    pub user_id: String, // Added for API compatibility
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Bid,  // Buy
    Ask,  // Sell
}

impl Order {
    pub fn new(id: u64, side: OrderSide, price: f64, quantity: f64, timestamp: u64, user_id: String) -> Self {
        Self {
            id,
            side,
            price: Price(price),
            quantity,
            timestamp,
            user_id,
        }
    }
}
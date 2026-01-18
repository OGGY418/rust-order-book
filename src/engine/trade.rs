use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub bid_order_id: u64,
    pub ask_order_id: u64,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

impl Trade {
    pub fn new(bid_order_id: u64, ask_order_id: u64, price: f64, quantity: f64, timestamp: u64) -> Self {
        Self {
            bid_order_id,
            ask_order_id,
            price,
            quantity,
            timestamp,
        }
    }

    pub fn get_trade_value(&self) -> f64 {
        self.price * self.quantity
    }
}
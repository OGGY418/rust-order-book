use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use crossbeam::queue::SegQueue;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::engine::order::{Order, OrderSide};
use crate::engine::price::Price;
use crate::engine::trade::Trade;


#[derive(Debug)]
pub struct OrderQueue {
    orders: DashMap<u64, Order>,
    order_queue: SegQueue<u64>,
    total_quantity: AtomicUsize,
}

impl OrderQueue {
    pub fn new() -> Self {
        Self {
            orders: DashMap::new(),
            order_queue: SegQueue::new(),
            total_quantity: AtomicUsize::new(0),
        }
    }

    pub fn add_order(&self, order: Order) {
        let quantity = (order.quantity * 1_000_000.0) as usize;
        self.orders.insert(order.id, order.clone());
        self.order_queue.push(order.id);
        self.total_quantity.fetch_add(quantity, Ordering::Relaxed);
    }

    pub fn remove_order(&self, order_id: u64) -> Option<Order> {
        if let Some((_, order)) = self.orders.remove(&order_id) {
            let quantity = (order.quantity * 1_000_000.0) as usize;
            self.total_quantity.fetch_sub(quantity, Ordering::Relaxed);
            Some(order)
        } else {
            None
        }
    }

    pub fn update_order(&self, order_id: u64, new_quantity: f64) -> bool {
        if let Some(mut order_ref) = self.orders.get_mut(&order_id) {
            let old_quantity = (order_ref.quantity * 1_000_000.0) as usize;
            let new_quantity_int = (new_quantity * 1_000_000.0) as usize;
            
            order_ref.quantity = new_quantity;
            self.total_quantity.fetch_add(new_quantity_int, Ordering::Relaxed);
            self.total_quantity.fetch_sub(old_quantity, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn get_total_quantity(&self) -> f64 {
        (self.total_quantity.load(Ordering::Relaxed) as f64) / 1_000_000.0
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    pub fn len(&self) -> usize {
        self.orders.len()
    }

    pub fn get_first_order(&self) -> Option<Order> {
        let mut temp_queue = Vec::new();
        let mut first_order = None;
        
        while let Some(order_id) = self.order_queue.pop() {
            if let Some(order) = self.orders.get(&order_id) {
                first_order = Some(order.clone());
                temp_queue.push(order_id);
                break;
            }
            temp_queue.push(order_id);
        }
        
        for order_id in temp_queue {
            self.order_queue.push(order_id);
        }
        
        first_order
    }

    pub fn remove_first_order(&self) -> Option<Order> {
        while let Some(order_id) = self.order_queue.pop() {
            if let Some(order) = self.remove_order(order_id) {
                return Some(order);
            }
        }
        None
    }

    pub fn get_order(&self, order_id: u64) -> Option<Order> {
        self.orders.get(&order_id).map(|o| o.clone())
    }
}


#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: Price,
    pub orders: Arc<OrderQueue>,
}

impl PriceLevel {
    pub fn new(price: f64) -> Self {
        Self {
            price: Price(price),
            orders: Arc::new(OrderQueue::new()),
        }
    }

    pub fn add_order(&self, order: Order) {
        self.orders.add_order(order);
    }

    pub fn remove_order(&self, order_id: u64) -> Option<Order> {
        self.orders.remove_order(order_id)
    }

    pub fn update_order(&self, order_id: u64, new_quantity: f64) -> bool {
        self.orders.update_order(order_id, new_quantity)
    }

    pub fn get_total_quantity(&self) -> f64 {
        self.orders.get_total_quantity()
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    pub fn len(&self) -> usize {
        self.orders.len()
    }

    pub fn get_first_order(&self) -> Option<Order> {
        self.orders.get_first_order()
    }

    pub fn remove_first_order(&self) -> Option<Order> {
        self.orders.remove_first_order()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookStats {
    pub total_orders_created: u64,
    pub total_orders_matched: u64,
    pub total_orders_cancelled: u64,
    pub total_volume_traded: f64,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub spread: Option<f64>,
    pub mid_price: Option<f64>,
    pub last_match_time: Option<u64>,
}

impl OrderBookStats {
    pub fn new() -> Self {
        Self {
            total_orders_created: 0,
            total_orders_matched: 0,
            total_orders_cancelled: 0,
            total_volume_traded: 0.0,
            best_bid: None,
            best_ask: None,
            spread: None,
            mid_price: None,
            last_match_time: None,
        }
    }

    pub fn update_market_data(&mut self, best_bid: Option<f64>, best_ask: Option<f64>) {
        self.best_bid = best_bid;
        self.best_ask = best_ask;
        
        if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
            self.spread = Some(ask - bid);
            self.mid_price = Some((bid + ask) / 2.0);
        } else {
            self.spread = None;
            self.mid_price = None;
        }
    }
}


#[derive(Debug)]
pub struct OrderBook {
    bids: RwLock<BTreeMap<Price, PriceLevel>>,
    asks: RwLock<BTreeMap<Price, PriceLevel>>,
    next_order_id: AtomicU64,
    stats: Arc<RwLock<OrderBookStats>>,
    matching_lock: parking_lot::Mutex<()>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: RwLock::new(BTreeMap::new()),
            asks: RwLock::new(BTreeMap::new()),
            next_order_id: AtomicU64::new(1),
            stats: Arc::new(RwLock::new(OrderBookStats::new())),
            matching_lock: parking_lot::Mutex::new(()),
        }
    }

 
    pub fn add_order(&self, side: OrderSide, price: f64, quantity: f64, timestamp: u64, user_id: String) -> (u64, Vec<Trade>) {
        let order_id = self.next_order_id.fetch_add(1, Ordering::Relaxed);
        let mut order = Order::new(order_id, side.clone(), price, quantity, timestamp, user_id);
        
        
        let trades = self.match_order(&mut order);
        
        if order.quantity > 0.0 {
            match side {
                OrderSide::Bid => {
                    let mut bids = self.bids.write();
                    bids.entry(Price(price))
                        .or_insert_with(|| PriceLevel::new(price))
                        .add_order(order);
                }
                OrderSide::Ask => {
                    let mut asks = self.asks.write();
                    asks.entry(Price(price))
                        .or_insert_with(|| PriceLevel::new(price))
                        .add_order(order);
                }
            }
        }

        {
            let mut stats = self.stats.write();
            stats.total_orders_created += 1;
            if !trades.is_empty() {
                stats.total_orders_matched += trades.len() as u64;
                stats.total_volume_traded += trades.iter().map(|t| t.price * t.quantity).sum::<f64>();
                stats.last_match_time = Some(timestamp);
            }
            self.update_stats_internal(&mut stats);
        }

        (order_id, trades)
    }

    fn match_order(&self, order: &mut Order) -> Vec<Trade> {
        let _lock = self.matching_lock.lock();
        let mut trades = Vec::new();

        match order.side {
            OrderSide::Bid => {
                
                loop {
                    let best_ask = self.get_best_ask();
                    if best_ask.is_none() || order.quantity <= 0.0 {
                        break;
                    }

                    let ask_price = best_ask.unwrap();
                    if order.price.as_f64() < ask_price {
                        break; 
                    }

                    let mut asks = self.asks.write();
                    if let Some(ask_level) = asks.get_mut(&Price(ask_price)) {
                        if let Some(ask_order) = ask_level.get_first_order() {
                            let trade_quantity = order.quantity.min(ask_order.quantity);
                            
                            trades.push(Trade::new(
                                order.id,
                                ask_order.id,
                                ask_price,
                                trade_quantity,
                                std::cmp::min(order.timestamp, ask_order.timestamp),
                            ));

                            order.quantity -= trade_quantity;

                            if ask_order.quantity <= trade_quantity {
                                ask_level.remove_first_order();
                            } else {
                                ask_level.update_order(ask_order.id, ask_order.quantity - trade_quantity);
                            }

                            if ask_level.is_empty() {
                                asks.remove(&Price(ask_price));
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            OrderSide::Ask => {
                
                loop {
                    let best_bid = self.get_best_bid();
                    if best_bid.is_none() || order.quantity <= 0.0 {
                        break;
                    }

                    let bid_price = best_bid.unwrap();
                    if order.price.as_f64() > bid_price {
                        break; 
                    }

                    let mut bids = self.bids.write();
                    if let Some(bid_level) = bids.get_mut(&Price(bid_price)) {
                        if let Some(bid_order) = bid_level.get_first_order() {
                            let trade_quantity = order.quantity.min(bid_order.quantity);
                            
                            trades.push(Trade::new(
                                bid_order.id,
                                order.id,
                                bid_price,
                                trade_quantity,
                                std::cmp::min(order.timestamp, bid_order.timestamp),
                            ));

                            order.quantity -= trade_quantity;

                            if bid_order.quantity <= trade_quantity {
                                bid_level.remove_first_order();
                            } else {
                                bid_level.update_order(bid_order.id, bid_order.quantity - trade_quantity);
                            }

                            if bid_level.is_empty() {
                                bids.remove(&Price(bid_price));
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        trades
    }

    pub fn remove_order(&self, order_id: u64, user_id: &str) -> Option<Order> {
        let mut removed_order = None;

        {
            let mut bids = self.bids.write();
            for (price, price_level) in bids.iter_mut() {
                if let Some(order) = price_level.orders.get_order(order_id) {
                    if order.user_id == user_id {
                        removed_order = price_level.remove_order(order_id);
                        if price_level.is_empty() {
                            let price_to_remove = price.clone();
                            drop(price_level);
                            bids.remove(&price_to_remove);
                        }
                        break;
                    }
                }
            }
        }

        if removed_order.is_none() {
            let mut asks = self.asks.write();
            for (price, price_level) in asks.iter_mut() {
                if let Some(order) = price_level.orders.get_order(order_id) {
                    if order.user_id == user_id {
                        removed_order = price_level.remove_order(order_id);
                        if price_level.is_empty() {
                            let price_to_remove = price.clone();
                            drop(price_level);
                            asks.remove(&price_to_remove);
                        }
                        break;
                    }
                }
            }
        }

        if removed_order.is_some() {
            let mut stats = self.stats.write();
            stats.total_orders_cancelled += 1;
            self.update_stats_internal(&mut stats);
        }

        removed_order
    }

    pub fn get_best_bid(&self) -> Option<f64> {
        let bids = self.bids.read();
        bids.keys().next_back().map(|p| p.as_f64())
    }

    pub fn get_best_ask(&self) -> Option<f64> {
        let asks = self.asks.read();
        asks.keys().next().map(|p| p.as_f64())
    }

    pub fn get_spread(&self) -> Option<f64> {
        let stats = self.stats.read();
        stats.spread
    }

    pub fn get_market_depth(&self, levels: usize) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        let bids: Vec<(f64, f64)> = {
            let bids = self.bids.read();
            bids.iter()
                .rev()
                .take(levels)
                .map(|(price, level)| (price.as_f64(), level.get_total_quantity()))
                .collect()
        };

        let asks: Vec<(f64, f64)> = {
            let asks = self.asks.read();
            asks.iter()
                .take(levels)
                .map(|(price, level)| (price.as_f64(), level.get_total_quantity()))
                .collect()
        };

        (bids, asks)
    }

    pub fn get_stats(&self) -> OrderBookStats {
        self.stats.read().clone()
    }

    fn update_stats_internal(&self, stats: &mut OrderBookStats) {
        let best_bid = self.get_best_bid();
        let best_ask = self.get_best_ask();
        stats.update_market_data(best_bid, best_ask);
    }

    pub fn clear(&self) {
        let mut bids = self.bids.write();
        let mut asks = self.asks.write();
        bids.clear();
        asks.clear();
        
        let mut stats = self.stats.write();
        *stats = OrderBookStats::new();
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}
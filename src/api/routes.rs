use std::sync::Arc;
use actix_web::{delete, get, post, web::{Data, Json}, HttpResponse, Responder};
use crate::engine::orderbook::OrderBook;
use crate::engine::order::OrderSide;
use crate::api::types::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[get("/depth")]
pub async fn get_depth(orderbook: Data<Arc<OrderBook>>) -> impl Responder {
    let (bids, asks) = orderbook.get_market_depth(20);
    
    let response = DepthResponse {
        bids: bids.into_iter()
            .map(|(price, quantity)| DepthLevel { price, quantity })
            .collect(),
        asks: asks.into_iter()
            .map(|(price, quantity)| DepthLevel { price, quantity })
            .collect(),
    };
    
    HttpResponse::Ok().json(response)
}

#[post("/order")]
pub async fn create_order(
    orderbook: Data<Arc<OrderBook>>,
    order: Json<CreateOrderRequest>,
) -> impl Responder {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let side = match order.side {
        Side::Buy => OrderSide::Bid,
        Side::Sell => OrderSide::Ask,
    };
    
    
    let (order_id, trades) = orderbook.add_order(
        side,
        order.price,
        order.quantity,
        timestamp,
        order.user_id.clone(),
    );
    
    
    let filled_quantity: f64 = trades.iter().map(|t| t.quantity).sum();
    let remaining_quantity = order.quantity - filled_quantity;
    
    
    let total_value: f64 = trades.iter().map(|t| t.price * t.quantity).sum();
    let average_price = if filled_quantity > 0.0 {
        total_value / filled_quantity
    } else {
        0.0
    };
    
    
    let status = if filled_quantity == 0.0 {
        OrderStatus::New
    } else if remaining_quantity > 0.0 {
        OrderStatus::PartiallyFilled
    } else {
        OrderStatus::Filled
    };
    
    
    let fills: Vec<Fill> = trades.iter().map(|t| t.into()).collect();
    
    let response = CreateOrderResponse {
        order_id: order_id.to_string(),
        filled_quantity,
        remaining_quantity,
        average_price,
        fills,
        status,
    };
    
    HttpResponse::Ok().json(response)
}

#[delete("/order")]
pub async fn delete_order(
    orderbook: Data<Arc<OrderBook>>,
    request: Json<DeleteOrderRequest>,
) -> impl Responder {
    let order_id: u64 = match request.order_id.parse() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid order_id"),
    };
    
    match orderbook.remove_order(order_id, &request.user_id) {
        Some(order) => {
            let response = DeleteOrderResponse {
                success: true,
                remaining_quantity: order.quantity,
                filled_quantity: 0.0, 
            };
            HttpResponse::Ok().json(response)
        }
        None => {
            let response = DeleteOrderResponse {
                success: false,
                remaining_quantity: 0.0,
                filled_quantity: 0.0,
            };
            HttpResponse::Ok().json(response)
        }
    }
}

#[get("/stats")]
pub async fn get_stats(orderbook: Data<Arc<OrderBook>>) -> impl Responder {
    let stats = orderbook.get_stats();
    HttpResponse::Ok().json(stats)
}

#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "order-book-hybrid"
    }))
}
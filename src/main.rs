mod matching_engine;
mod order;
mod order_book;

use axum::{
    extract::{State, Path},
    routing::{get, post, delete},
    Json, Router,
};

use matching_engine::MatchingEngine;
use order::{Order, OrderType, Side, Trade};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
struct NewOrderRequest {
    side: Side,
    order_type: OrderType,
    /// Price in cents; for market orders this can be omitted or 0
    price: Option<u64>,
    quantity: u64,
}

#[derive(Debug, Serialize)]
struct OrderBookView {
    bids: Vec<Order>,
    asks: Vec<Order>,
}

#[derive(Debug, Serialize)]
struct NewOrderResponse {
    trades: Vec<Trade>,
    orderbook: OrderBookView,
}

#[tokio::main]
async fn main() {
    let engine = MatchingEngine::new();

    let app = Router::new()
        .route("/orderbook", get(get_orderbook))
        .route("/orders", post(post_order))
        .route("/orders/{id}/cancel", delete(cancel_order))
        .with_state(engine);

    let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 61666));
    println!("Starting server on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_orderbook(State(engine): State<MatchingEngine>) -> Json<OrderBookView> {
    let bids = engine.get_buy_orders().await;
    let asks = engine.get_sell_orders().await;
    Json(OrderBookView { bids, asks })
}

async fn post_order(
    State(engine): State<MatchingEngine>,
    Json(req): Json<NewOrderRequest>,
) -> Json<NewOrderResponse> {
    let id = engine.next_id().await;
    let ts = engine.next_timestamp().await;
    let price = match req.order_type {
        OrderType::Limit => req.price.unwrap_or(0),
        OrderType::Market => 0,
    };
    let order = Order::new(id, req.side, req.order_type, price, req.quantity, ts);
    let trades = engine.submit_order(order).await;
    let bids = engine.get_buy_orders().await;
    let asks = engine.get_sell_orders().await;
    Json(NewOrderResponse {
        trades,
        orderbook: OrderBookView { bids, asks },
    })
}

async fn cancel_order(
    State(engine): State<MatchingEngine>,
    Path(order_id): Path<u64>,
) -> Json<CancelResponse> {
    let result = engine.cancel_order(order_id).await;
    Json(CancelResponse { result })
}

#[derive(Debug, Serialize)]
struct CancelResponse {
    result: matching_engine::CancelResult,
}

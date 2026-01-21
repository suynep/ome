mod matchingengine;
mod order;
mod orderbook;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};

use chrono::{DateTime, Utc};
use matchingengine::MatchingEngine;
use order::{Order, OrderType, Side, Trade};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum PriceType {
    Unsigned(u64),
    Float(f64),
}

#[derive(Debug, Deserialize)]
struct NewOrderRequest {
    side: Side,
    order_type: OrderType,
    /// Price in cents; for market orders this can be omitted or 0
    price: Option<PriceType>,
    quantity: u64,
}

#[derive(Debug, Serialize)]
struct OrderBookView {
    bids: Vec<Order>,
    asks: Vec<Order>,
}

#[derive(Debug, Serialize)]
struct NewOrderResponse {
    // trades: Vec<Trade>,
    // orderbook: OrderBookView,
    id: String,
    trades: Option<Vec<Trade>>,
}

#[derive(Debug, Serialize)]
struct CancelResponse {
    result: bool,
}

#[derive(Debug, Serialize)]
struct AllTradesResponse {
    trades: Vec<Trade>,
}

#[tokio::main]
async fn main() {
    let engine = MatchingEngine::new();

    let app = Router::new()
        .route("/orderbook", get(get_orderbook))
        .route("/orders", post(post_order))
        .route("/orders/{id}/cancel", delete(cancel_order))
        .route("/trades", get(get_all_trades))
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
    State(mut engine): State<MatchingEngine>,
    Json(req): Json<NewOrderRequest>,
) -> Json<NewOrderResponse> {
    let id = uuid::Uuid::new_v4().to_string();
    let utc_datetime: DateTime<Utc> = Utc::now();
    let ts = utc_datetime.timestamp_nanos_opt().unwrap_or(0);
    let price = match req.order_type {
        OrderType::Limit => {
            let price = req.price.unwrap_or(PriceType::Unsigned(0));
            match price {
                PriceType::Float(f) => (f * 100.0) as u64,
                PriceType::Unsigned(u) => u,
            }
            // req.price.unwrap_or(0)
        }
        OrderType::Market => 0,
    };
    let order = Order::new(
        id,
        req.side,
        req.order_type,
        req.quantity,
        price,
        ts.try_into().unwrap(),
    );

    let trades = engine.submit_order(order.clone()).await;

    // let bids = engine.get_buy_orders().await;
    // let asks = engine.get_sell_orders().await;
    if trades.len() == 0 {
        Json(NewOrderResponse {
            id: order.id,
            trades: None,
            // orderbook: OrderBookView { bids, asks },
        })
    } else {
        Json(NewOrderResponse {
            id: order.id,
            trades: Some(trades),
            // orderbook: OrderBookView { bids, asks },
        })
    }
}

async fn cancel_order(
    State(mut engine): State<MatchingEngine>,
    Path(order_id): Path<String>,
) -> Json<CancelResponse> {
    let result = engine.cancel_order(order_id).await;
    Json(CancelResponse { result })
}

async fn get_all_trades(State(engine): State<MatchingEngine>) -> Json<AllTradesResponse> {
    let trades_guard = engine.trades.read().await;
    let trades_vec: Vec<Trade> = trades_guard
        .iter()
        .map(|arc_trade| (*arc_trade).clone())
        .collect();
    Json(AllTradesResponse { trades: trades_vec })
}

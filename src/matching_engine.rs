use crate::order::{Order, OrderId, OrderType, Side, Timestamp, Trade};
use crate::order_book::OrderBook;

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MatchingEngine {
    order_book: Arc<RwLock<OrderBook>>,
    trades: Arc<RwLock<Vec<Trade>>>,
    next_order_id: Arc<RwLock<OrderId>>,
    current_timestamp: Arc<RwLock<u64>>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        MatchingEngine {
            order_book: Arc::new(RwLock::new(OrderBook::new())),
            trades: Arc::new(RwLock::new(Vec::<Trade>::new())),
            next_order_id: Arc::new(RwLock::new(1)), // starts from 1 (base id)
            current_timestamp: Arc::new(RwLock::new(1)), // can be replaced w/ actual timestamps
                                                     // later (since posix timestamps are basically
                                                     // u64 ints)
        }
    }

    pub async fn next_id(&self) -> OrderId {
        let mut id = self.next_order_id.write().await;
        let current = *id;
        *id += 1; // allowed since RwLock.write is used (provides a temporary write atomic)
        current
    }

    pub async fn next_timestamp(&self) -> Timestamp {
        // this is for demo purposes ONLY, in prod
        // env, you'll set the CURRENT time as the
        // timestamp
        let mut timestamp = self.current_timestamp.write().await;
        let current = *timestamp;
        *timestamp += 1; // 
        current
    }

    pub async fn submit_order(&self, mut order: Order) -> Vec<Trade> {
        let mut new_trades = Vec::new();
        let mut order_book = self.order_book.write().await;

        loop {
            let best_opposing = match order.side {
                Side::Buy => order_book.peek_best_sell(),
                Side::Sell => order_book.peek_best_buy(),
            };

            let best_opposing = match best_opposing {
                Some(o) => o,
                None => break,
            };

            if !order.can_match(&best_opposing) {
                break;
            }

            let execution_price = match (order.order_type, best_opposing.order_type) {
                (OrderType::Market, _) => best_opposing.price,
                (_, OrderType::Market) => order.price,

                (OrderType::Limit, OrderType::Limit) => best_opposing.price,
            };

            let trade_quantity = order.quantity.min(best_opposing.quantity);

            let mut opposing_order = match order.side {
                Side::Buy => order_book.pop_best_sell().unwrap(),
                Side::Sell => order_book.pop_best_buy().unwrap(),
            };

            let trade = match order.side {
                Side::Buy => {
                    Trade::new(order.id, opposing_order.id, execution_price, trade_quantity)
                }
                Side::Sell => {
                    Trade::new(opposing_order.id, order.id, execution_price, trade_quantity)
                }
            };

            new_trades.push(trade);

            order.quantity -= trade_quantity;
            opposing_order.quantity -= trade_quantity;

            if opposing_order.quantity > 0 {
                order_book.add_order(opposing_order);
            }

            if order.quantity == 0 {
                break;
            }
        }
        // if there's remaining order and it's a limit order, add it to the book
        if order.quantity > 0 && order.order_type == OrderType::Limit {
            order_book.add_order(order);
        }
        {
            let mut trades = self.trades.write().await;
            trades.extend(new_trades.clone());
        }

        new_trades
    }
    /// Returns the current state of the order book (all active buy orders)
    pub async fn get_buy_orders(&self) -> Vec<Order> {
        let order_book = self.order_book.read().await;
        order_book.get_buy_orders()
    }

    /// Returns the current state of the order book (all active sell orders)
    pub async fn get_sell_orders(&self) -> Vec<Order> {
        let order_book = self.order_book.read().await;
        order_book.get_sell_orders()
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MatchingEngine {
    fn clone(&self) -> Self {
        MatchingEngine {
            order_book: Arc::clone(&self.order_book),
            trades: Arc::clone(&self.trades),
            next_order_id: Arc::clone(&self.next_order_id),
            current_timestamp: Arc::clone(&self.current_timestamp),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::OrderType;

    #[tokio::test]
    async fn test_matching_engine_creation() {
        let engine = MatchingEngine::new();
        let buy_orders = engine.get_buy_orders().await;
        let sell_orders = engine.get_sell_orders().await;
        
        assert_eq!(buy_orders.len(), 0);
        assert_eq!(sell_orders.len(), 0);
    }

    #[tokio::test]
    async fn test_order_id_generation() {
        let engine = MatchingEngine::new();
        let id1 = engine.next_id().await;
        let id2 = engine.next_id().await;
        let id3 = engine.next_id().await;
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[tokio::test]
    async fn test_timestamp_generation() {
        let engine = MatchingEngine::new();
        let ts1 = engine.next_timestamp().await;
        let ts2 = engine.next_timestamp().await;
        let ts3 = engine.next_timestamp().await;
        
        assert_eq!(ts1, 1);
        assert_eq!(ts2, 2);
        assert_eq!(ts3, 3);
    }

    #[tokio::test]
    async fn test_limit_order_matching() {
        let engine = MatchingEngine::new();
        
        // Submit a sell order
        let sell_order = Order::new(1, Side::Sell, OrderType::Limit, 1000, 100, 1);
        engine.submit_order(sell_order).await;
        
        // Submit a matching buy order
        let buy_order = Order::new(2, Side::Buy, OrderType::Limit, 1050, 100, 2);
        let trades = engine.submit_order(buy_order).await;
        
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].buy_order_id, 2);
        assert_eq!(trades[0].sell_order_id, 1);
        assert_eq!(trades[0].price, 1000);
        assert_eq!(trades[0].quantity, 100);
        
        let buy_orders = engine.get_buy_orders().await;
        let sell_orders = engine.get_sell_orders().await;
        assert_eq!(buy_orders.len(), 0);
        assert_eq!(sell_orders.len(), 0);
    }

    #[tokio::test]
    async fn test_partial_fill() {
        let engine = MatchingEngine::new();
        
        // Submit a large sell order
        let sell_order = Order::new(1, Side::Sell, OrderType::Limit, 1000, 200, 1);
        engine.submit_order(sell_order).await;
        
        // Submit a smaller matching buy order
        let buy_order = Order::new(2, Side::Buy, OrderType::Limit, 1050, 100, 2);
        let trades = engine.submit_order(buy_order).await;
        
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 100);
        
        // Check that remaining sell order is in the book
        let sell_orders = engine.get_sell_orders().await;
        assert_eq!(sell_orders.len(), 1);
        assert_eq!(sell_orders[0].quantity, 100);
    }

    #[tokio::test]
    async fn test_limit_order_no_match() {
        let engine = MatchingEngine::new();
        
        // Submit a sell order
        let sell_order = Order::new(1, Side::Sell, OrderType::Limit, 1000, 100, 1);
        engine.submit_order(sell_order).await;
        
        // Submit a non-matching buy order (price too low)
        let buy_order = Order::new(2, Side::Buy, OrderType::Limit, 900, 100, 2);
        let trades = engine.submit_order(buy_order).await;
        
        assert_eq!(trades.len(), 0);
        
        let buy_orders = engine.get_buy_orders().await;
        let sell_orders = engine.get_sell_orders().await;
        assert_eq!(buy_orders.len(), 1);
        assert_eq!(sell_orders.len(), 1);
    }
}

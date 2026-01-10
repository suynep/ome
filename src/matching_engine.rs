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
    use crate::order::Order;

    #[tokio::test]
    async fn test_simple_match() {
        let engine = AsyncMatchingEngine::new();

        // Add a sell order
        let sell_order = Order::new(1, Side::Sell, OrderType::Limit, 1000, 100, 1);
        engine.submit_order(sell_order).await;

        // Add a matching buy order
        let buy_order = Order::new(2, Side::Buy, OrderType::Limit, 1000, 100, 2);
        let trades = engine.submit_order(buy_order).await;

        // Should have executed one trade
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 100);
        assert_eq!(trades[0].price, 1000);

        // Order book should be empty
        assert_eq!(engine.active_buy_count().await, 0);
        assert_eq!(engine.active_sell_count().await, 0);
    }

    #[tokio::test]
    async fn test_partial_fill() {
        let engine = AsyncMatchingEngine::new();

        // Add a large sell order
        let sell_order = Order::new(1, Side::Sell, OrderType::Limit, 1000, 200, 1);
        engine.submit_order(sell_order).await;

        // Add a smaller buy order
        let buy_order = Order::new(2, Side::Buy, OrderType::Limit, 1000, 100, 2);
        let trades = engine.submit_order(buy_order).await;

        // Should have executed one trade
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 100);

        // Sell order should have 100 remaining
        assert_eq!(engine.active_sell_count().await, 1);
        let remaining_sells = engine.get_sell_orders().await;
        assert_eq!(remaining_sells[0].quantity, 100);
    }

    #[tokio::test]
    async fn test_price_priority() {
        let engine = AsyncMatchingEngine::new();

        // Add two sell orders at different prices
        let sell1 = Order::new(1, Side::Sell, OrderType::Limit, 1100, 100, 1);
        let sell2 = Order::new(2, Side::Sell, OrderType::Limit, 1000, 100, 2);
        engine.submit_order(sell1).await;
        engine.submit_order(sell2).await;

        // Add a buy order that can match both
        let buy_order = Order::new(3, Side::Buy, OrderType::Limit, 1100, 100, 3);
        let trades = engine.submit_order(buy_order).await;

        // Should match with the cheaper sell order (order 2)
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].sell_order_id, 2);
        assert_eq!(trades[0].price, 1000);
    }

    #[tokio::test]
    async fn test_time_priority() {
        let engine = AsyncMatchingEngine::new();

        // Add two sell orders at the same price
        let sell1 = Order::new(1, Side::Sell, OrderType::Limit, 1000, 100, 1);
        let sell2 = Order::new(2, Side::Sell, OrderType::Limit, 1000, 100, 2);
        engine.submit_order(sell1).await;
        engine.submit_order(sell2).await;

        // Add a buy order
        let buy_order = Order::new(3, Side::Buy, OrderType::Limit, 1000, 100, 3);
        let trades = engine.submit_order(buy_order).await;

        // Should match with the earlier order (order 1)
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].sell_order_id, 1);
    }

    #[tokio::test]
    async fn test_market_order() {
        let engine = AsyncMatchingEngine::new();

        // Add a sell limit order
        let sell_order = Order::new(1, Side::Sell, OrderType::Limit, 1000, 100, 1);
        engine.submit_order(sell_order).await;

        // Add a market buy order (price doesn't matter)
        let buy_order = Order::new(2, Side::Buy, OrderType::Market, 0, 100, 2);
        let trades = engine.submit_order(buy_order).await;

        // Should execute at the sell order's price
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 1000);
    }

    #[tokio::test]
    async fn test_no_match_when_prices_dont_cross() {
        let engine = AsyncMatchingEngine::new();

        // Add a sell order at $11
        let sell_order = Order::new(1, Side::Sell, OrderType::Limit, 1100, 100, 1);
        engine.submit_order(sell_order).await;

        // Add a buy order at $10 (doesn't cross)
        let buy_order = Order::new(2, Side::Buy, OrderType::Limit, 1000, 100, 2);
        let trades = engine.submit_order(buy_order).await;

        // Should not execute any trades
        assert_eq!(trades.len(), 0);

        // Both orders should remain in the book
        assert_eq!(engine.active_buy_count().await, 1);
        assert_eq!(engine.active_sell_count().await, 1);
    }

    #[tokio::test]
    async fn test_cancel_order() {
        let engine = AsyncMatchingEngine::new();

        let order = Order::new(1, Side::Buy, OrderType::Limit, 1000, 100, 1);
        engine.submit_order(order).await;

        assert_eq!(engine.active_buy_count().await, 1);

        // Cancel the order
        assert!(engine.cancel_order(1).await);
        assert_eq!(engine.active_buy_count().await, 0);

        // Try to cancel again
        assert!(!engine.cancel_order(1).await);
    }

    #[tokio::test]
    async fn test_pdf_example() {
        // This test reproduces the exact example from MatchingEngine.pdf
        let engine = AsyncMatchingEngine::new();

        // Initial order book state
        // Buy 100 shares at $9.50 (timestamp 1)
        let buy1 = Order::new(1, Side::Buy, OrderType::Limit, 950, 100, 1);
        engine.submit_order(buy1).await;

        // Buy 200 shares at $9.00 (timestamp 2)
        let buy2 = Order::new(2, Side::Buy, OrderType::Limit, 900, 200, 2);
        engine.submit_order(buy2).await;

        // Sell 150 shares at $10.50 (timestamp 3)
        let sell1 = Order::new(3, Side::Sell, OrderType::Limit, 1050, 150, 3);
        engine.submit_order(sell1).await;

        // Sell 100 shares at $10.00 (timestamp 4)
        let sell2 = Order::new(4, Side::Sell, OrderType::Limit, 1000, 100, 4);
        engine.submit_order(sell2).await;

        // Verify initial state
        assert_eq!(engine.active_buy_count().await, 2);
        assert_eq!(engine.active_sell_count().await, 2);

        // New buy limit order: Buy 150 shares at $10.50 (timestamp 5)
        let new_buy = Order::new(5, Side::Buy, OrderType::Limit, 1050, 150, 5);
        let trades = engine.submit_order(new_buy).await;

        // Should execute 2 trades as described in the PDF:
        // 1. 100 shares at $10.00 (matching with sell order 4)
        // 2. 50 shares at $10.50 (matching with sell order 3)
        assert_eq!(trades.len(), 2);

        // First trade: 100 shares at $10.00
        assert_eq!(trades[0].buy_order_id, 5);
        assert_eq!(trades[0].sell_order_id, 4);
        assert_eq!(trades[0].quantity, 100);
        assert_eq!(trades[0].price, 1000); // $10.00

        // Second trade: 50 shares at $10.50
        assert_eq!(trades[1].buy_order_id, 5);
        assert_eq!(trades[1].sell_order_id, 3);
        assert_eq!(trades[1].quantity, 50);
        assert_eq!(trades[1].price, 1050); // $10.50

        // Final state check
        // Buy orders: still have the original 2 buy orders (at $9.50 and $9.00)
        assert_eq!(engine.active_buy_count().await, 2);

        // Sell orders: only 1 remaining (100 shares at $10.50 from order 3)
        assert_eq!(engine.active_sell_count().await, 1);
        let sell_orders = engine.get_sell_orders().await;
        assert_eq!(sell_orders[0].id, 3);
        assert_eq!(sell_orders[0].quantity, 100); // 150 - 50 = 100 remaining
        assert_eq!(sell_orders[0].price, 1050);
    }

    #[tokio::test]
    async fn test_concurrent_submissions() {
        let engine = AsyncMatchingEngine::new();

        // Submit 3 sell orders concurrently
        let mut handles = vec![];

        for i in 0..3 {
            let engine_clone = engine.clone();
            let handle = tokio::spawn(async move {
                let sell = Order::new(
                    1 + i as u64,
                    Side::Sell,
                    OrderType::Limit,
                    1100,
                    50,
                    1 + i as u64,
                );
                engine_clone.submit_order(sell).await
            });
            handles.push(handle);
        }

        // Wait for all submissions
        for handle in handles {
            let _ = handle.await;
        }

        // Should have all 3 sell orders
        assert_eq!(engine.active_sell_count().await, 3);

        // Submit a buy order that matches all three
        let buy_order = Order::new(4, Side::Buy, OrderType::Limit, 1100, 150, 4);
        let trades = engine.submit_order(buy_order).await;

        // Should have 3 trades
        assert_eq!(trades.len(), 3);

        // All trades should be at $1100
        for trade in trades {
            assert_eq!(trade.price, 1100);
            assert_eq!(trade.quantity, 50);
        }

        // No remaining sell orders
        assert_eq!(engine.active_sell_count().await, 0);
    }

    #[tokio::test]
    async fn test_multiple_partial_fills() {
        let engine = AsyncMatchingEngine::new();

        // Add 3 small sell orders
        for i in 0..3 {
            let sell = Order::new(
                i as u64 + 1,
                Side::Sell,
                OrderType::Limit,
                1000,
                50,
                i as u64 + 1,
            );
            engine.submit_order(sell).await;
        }

        // Add a large buy order that matches all
        let buy_order = Order::new(4, Side::Buy, OrderType::Limit, 1000, 150, 4);
        let trades = engine.submit_order(buy_order).await;

        // Should have 3 trades
        assert_eq!(trades.len(), 3);

        // Each trade should be 50 units
        for trade in trades {
            assert_eq!(trade.quantity, 50);
            assert_eq!(trade.price, 1000);
        }

        // No remaining orders
        assert_eq!(engine.active_buy_count().await, 0);
        assert_eq!(engine.active_sell_count().await, 0);
    }

    #[tokio::test]
    async fn test_trade_history() {
        let engine = AsyncMatchingEngine::new();

        // Submit orders and verify all trades are recorded
        let sell_order = Order::new(1, Side::Sell, OrderType::Limit, 1000, 100, 1);
        engine.submit_order(sell_order).await;

        let buy_order = Order::new(2, Side::Buy, OrderType::Limit, 1000, 100, 2);
        engine.submit_order(buy_order).await;

        // Get trades from history
        let trades = engine.get_trades().await;
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].buy_order_id, 2);
        assert_eq!(trades[0].sell_order_id, 1);
    }

    #[tokio::test]
    async fn test_next_id_uniqueness() {
        let engine = AsyncMatchingEngine::new();

        let mut ids = vec![];
        for _ in 0..10 {
            ids.push(engine.next_id().await);
        }

        // All IDs should be unique
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 10);

        // IDs should be sequential
        for i in 0..10 {
            assert_eq!(ids[i], (i + 1) as u64);
        }
    }

    #[tokio::test]
    async fn test_next_timestamp_uniqueness() {
        let engine = AsyncMatchingEngine::new();

        let mut timestamps = vec![];
        for _ in 0..10 {
            timestamps.push(engine.next_timestamp().await);
        }

        // All timestamps should be unique and incremental
        for i in 0..10 {
            assert_eq!(timestamps[i], (i + 1) as u64);
        }
    }

    #[tokio::test]
    async fn test_market_sell_order() {
        let engine = AsyncMatchingEngine::new();

        // Add a buy limit order
        let buy_order = Order::new(1, Side::Buy, OrderType::Limit, 1000, 100, 1);
        engine.submit_order(buy_order).await;

        // Add a market sell order
        let sell_order = Order::new(2, Side::Sell, OrderType::Market, 0, 100, 2);
        let trades = engine.submit_order(sell_order).await;

        // Should execute at the buy order's price
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 1000);
        assert_eq!(trades[0].sell_order_id, 2);
        assert_eq!(trades[0].buy_order_id, 1);
    }

    #[tokio::test]
    async fn test_unmatched_limit_order_stays_in_book() {
        let engine = AsyncMatchingEngine::new();

        // Add a buy order that doesn't match anything
        let buy_order = Order::new(1, Side::Buy, OrderType::Limit, 900, 100, 1);
        let trades = engine.submit_order(buy_order).await;

        // No trades should execute
        assert_eq!(trades.len(), 0);

        // Order should remain in the book
        assert_eq!(engine.active_buy_count().await, 1);
        let buy_orders = engine.get_buy_orders().await;
        assert_eq!(buy_orders[0].id, 1);
        assert_eq!(buy_orders[0].quantity, 100);
    }

    #[tokio::test]
    async fn test_unmatched_market_order_doesnt_stay() {
        let engine = AsyncMatchingEngine::new();

        // Add a market buy order with no sell orders
        let buy_order = Order::new(1, Side::Buy, OrderType::Market, 0, 100, 1);
        let trades = engine.submit_order(buy_order).await;

        // No trades should execute
        assert_eq!(trades.len(), 0);

        // Market order should NOT remain in the book
        assert_eq!(engine.active_buy_count().await, 0);
    }

    #[tokio::test]
    async fn test_engine_cloning() {
        let engine1 = AsyncMatchingEngine::new();

        // Add an order to engine1
        let buy_order = Order::new(1, Side::Buy, OrderType::Limit, 1000, 100, 1);
        engine1.submit_order(buy_order).await;

        // Clone the engine
        let engine2 = engine1.clone();

        // Both should see the same state
        assert_eq!(engine1.active_buy_count().await, 1);
        assert_eq!(engine2.active_buy_count().await, 1);

        // Add an order to cloned engine
        let sell_order = Order::new(2, Side::Sell, OrderType::Limit, 1000, 100, 2);
        engine2.submit_order(sell_order).await;

        // Both should see the trade
        let trades1 = engine1.get_trades().await;
        let trades2 = engine2.get_trades().await;
        assert_eq!(trades1.len(), 1);
        assert_eq!(trades2.len(), 1);
    }
}

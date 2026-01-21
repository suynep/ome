use crate::{
    order::{Order, OrderId, OrderType, Side, Trade},
    orderbook::OrderBook,
};

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

pub const TRADE_POOL_SIZE: usize = 500; // defines the size of MatchingEngine::new().trades field

pub struct MatchingEngine {
    order_book: Arc<RwLock<OrderBook>>,
    pub trades: Arc<RwLock<VecDeque<Trade>>>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        MatchingEngine {
            order_book: Arc::new(RwLock::new(OrderBook::new())),
            trades: Arc::new(RwLock::new(VecDeque::<Trade>::with_capacity(
                TRADE_POOL_SIZE,
            ))),
        }
    }

    pub async fn submit_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut new_trades = Vec::<Trade>::new();
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
                (_, OrderType::Market) => order.price, // w/ assumption that market orders persist
                // in the orderbook (this is false, but
                // compiler complains abt exhaustion)
                (OrderType::Limit, OrderType::Limit) => best_opposing.price,
            };

            let trade_quantity = order.quantity.min(best_opposing.quantity);

            let mut opposing_order = match order.side {
                Side::Buy => order_book.pop_best_sell().unwrap(),
                Side::Sell => order_book.pop_best_buy().unwrap(),
            };

            let trade = match order.side {
                Side::Buy => Trade::new(
                    order.id.clone(),
                    opposing_order.id.clone(),
                    execution_price,
                    trade_quantity,
                ),
                Side::Sell => Trade::new(
                    opposing_order.id.clone(),
                    order.id.clone(),
                    execution_price,
                    trade_quantity,
                ),
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

        if order.quantity > 0 && order.order_type == OrderType::Limit {
            order_book.add_order(order);
        }

        {
            let mut trades = self.trades.write().await;
            for trade in new_trades.clone() {
                if trades.len() >= TRADE_POOL_SIZE {
                    trades.pop_front();
                }

                trades.push_back(trade);
            }
            // trades.extend(new_trades.clone());
        }

        new_trades
    }

    pub async fn cancel_order(&mut self, order_id: OrderId) -> bool {
        let mut order_book = self.order_book.write().await;
        order_book.cancel_order(order_id)
    }

    pub async fn get_buy_orders(&self) -> Vec<Order> {
        let order_book = self.order_book.write().await;
        order_book.get_buy_orders()
    }

    /// Returns the current state of the order book (all active sell orders)
    pub async fn get_sell_orders(&self) -> Vec<Order> {
        let order_book = self.order_book.write().await;
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
        }
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;

    use super::*;
    #[tokio::test]
    async fn test_submit_order() {
        let ob = OrderBook::new();
        let o1 = Order::new(String::from("1"), Side::Buy, OrderType::Limit, 2000, 10, 1);
        let o2 = Order::new(String::from("2"), Side::Buy, OrderType::Limit, 2000, 200, 2);
        let o3 = Order::new(String::from("3"), Side::Buy, OrderType::Limit, 2000, 200, 1);
        let o4 = Order::new(String::from("4"), Side::Buy, OrderType::Limit, 2000, 500, 1);
        let o5 = Order::new(
            String::from("5"),
            Side::Sell,
            OrderType::Limit,
            2000,
            500,
            1,
        );
        let o6 = Order::new(
            String::from("6"),
            Side::Sell,
            OrderType::Limit,
            2000,
            500,
            1,
        );

        let mut me = MatchingEngine::new();
        me.order_book = Arc::new(RwLock::new(ob));

        me.submit_order(o4).await;
        me.submit_order(o1).await;
        me.submit_order(o2).await;
        me.submit_order(o3).await;
        me.submit_order(o5).await;
        me.submit_order(o6).await;

        println!("{}", me.order_book.read().await);
        println!("{}", me.order_book.read().await);
        println!("{:?}", me.trades);
    }

    #[tokio::test]
    async fn test_market_orders() {
        let ob = OrderBook::new();
        let o1 = Order::new(String::from("1"), Side::Buy, OrderType::Market, 20, 100, 1);
        let o2 = Order::new(String::from("2"), Side::Buy, OrderType::Market, 200, 100, 2);

        let o3 = Order::new(String::from("3"), Side::Sell, OrderType::Limit, 10, 2000, 1);

        let mut me = MatchingEngine::new();
        me.order_book = Arc::new(RwLock::new(ob));

        me.submit_order(o3).await;
        me.submit_order(o1).await;
        me.submit_order(o2).await;

        println!("{}", me.order_book.read().await);
        println!("TRADES: {:?}", me.trades.read().await);
        println!("ORDER_MAP: {:?}", me.order_book.read().await.order_map);
    }

    #[tokio::test]
    async fn test_trade_pool_size_timestamp() {
        use rand::rng;
        let mut rng = rng();
        let mut engine = MatchingEngine::new();
        const BUY_MOCK_SIZE: usize = 1500000;
        const SELL_MOCK_SIZE: usize = 1500000;
        for i in 0..BUY_MOCK_SIZE {
            let price = rng.random_range(800..=1000);
            let quantity = rng.random_range(100..=200);
            let order = Order::new(
                format!("{i}"),
                Side::Buy,
                OrderType::Limit,
                quantity,
                price,
                i.try_into().unwrap(),
            );
            engine.submit_order(order).await;
        }

        for i in 0..SELL_MOCK_SIZE {
            let price = rng.random_range(800..=1000);
            let quantity = rng.random_range(100..=200);
            let order = Order::new(
                format!("{i}"),
                Side::Sell,
                OrderType::Limit,
                quantity,
                price,
                i.try_into().unwrap(),
            );
            engine.submit_order(order).await;
        }

        println!("{:?}", engine.trades.read().await);

        println!("\n{}", engine.trades.read().await.len());
    }
}

// the fun stuff: Order Book implementation
//
//

use crate::order::{compare_buy_orders, compare_sell_orders, Order, OrderId, Side};
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct OrderBook {
    bids: BTreeMap<u64, Vec<Order>>,
    asks: BTreeMap<u64, Vec<Order>>,
    orders_map: HashMap<OrderId, (Side, u64)>,
    canceled_orders: HashSet<OrderId>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::<u64, Vec<Order>>::new(),
            asks: BTreeMap::<u64, Vec<Order>>::new(),
            orders_map: HashMap::<OrderId, (Side, u64)>::new(),
            canceled_orders: HashSet::<OrderId>::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let price = order.price;
        let side = order.side;
        self.orders_map.insert(order.id, (side, price));
        match side {
            Side::Buy => {
                let q = self.bids.entry(price).or_insert_with(Vec::new);
                // Insert preserving timestamp ascending (earlier first)
                let pos = q
                    .iter()
                    .position(|o| o.timestamp > order.timestamp)
                    .unwrap_or(q.len());
                q.insert(pos, order);
            }
            Side::Sell => {
                let q = self.asks.entry(price).or_insert_with(Vec::new);
                let pos = q
                    .iter()
                    .position(|o| o.timestamp > order.timestamp)
                    .unwrap_or(q.len());
                q.insert(pos, order);
            }
        }
    }

    // the following peek functions return the best buy/sell without removing them from the heap
    pub fn peek_best_buy(&mut self) -> Option<Order> {
        loop {
            let (best_price, _) = match self.bids.last_key_value() {
                Some((p, q)) => (*p, q),
                None => return None,
            };

            if let Some(q) = self.bids.get_mut(&best_price) {
                while let Some(front) = q.first() {
                    if self.canceled_orders.contains(&front.id) {
                        let removed = q.remove(0);
                        self.orders_map.remove(&removed.id);
                        continue;
                    }
                    return Some(front.clone());
                }

                if q.is_empty() {
                    self.bids.remove(&best_price);
                    continue;
                }
            }
        }
    }

    pub fn peek_best_sell(&mut self) -> Option<Order> {
        loop {
            let (best_price, _) = match self.asks.first_key_value() {
                Some((p, q)) => (*p, q),
                None => return None,
            };

            if let Some(q) = self.asks.get_mut(&best_price) {
                while let Some(front) = q.first() {
                    if self.canceled_orders.contains(&front.id) {
                        let removed = q.remove(0);
                        self.orders_map.remove(&removed.id);
                        continue;
                    }
                    return Some(front.clone());
                }
                if q.is_empty() {
                    self.asks.remove(&best_price);
                    continue;
                }
            }
        }
    }

    pub fn pop_best_buy(&mut self) -> Option<Order> {
        loop {
            let best_price = match self.bids.last_key_value() {
                Some((p, _)) => *p,
                None => return None,
            };

            if let Some(q) = self.bids.get_mut(&best_price) {
                while let Some(front) = q.first() {
                    if self.canceled_orders.contains(&front.id) {
                        let removed = q.remove(0);
                        self.orders_map.remove(&removed.id);
                        continue;
                    }
                    let popped = q.remove(0);
                    self.orders_map.remove(&popped.id);
                    if q.is_empty() {
                        self.bids.remove(&best_price);
                    }

                    return Some(popped);
                }

                self.bids.remove(&best_price);
            }
        }
    }

    pub fn pop_best_sell(&mut self) -> Option<Order> {
        loop {
            let best_price = match self.asks.first_key_value() {
                Some((p, _)) => *p,
                None => return None,
            };

            if let Some(q) = self.asks.get_mut(&best_price) {
                while let Some(front) = q.first() {
                    if self.canceled_orders.contains(&front.id) {
                        let removed = q.remove(0);
                        self.orders_map.remove(&removed.id);
                        continue;
                    }
                    let popped = q.remove(0);
                    self.orders_map.remove(&popped.id);
                    if q.is_empty() {
                        self.asks.remove(&best_price);
                    }
                    return Some(popped);
                }

                self.asks.remove(&best_price);
            }
        }
    }
    /// Returns all active buy orders sorted by priority (best first)
    pub fn get_buy_orders(&self) -> Vec<Order> {
        let mut orders: Vec<Order> = Vec::new();
        for (_p, q) in self.bids.iter().rev() {
            for o in q.iter() {
                if !self.canceled_orders.contains(&o.id) {
                    orders.push(o.clone());
                }
            }
        }
        orders.sort_by(compare_buy_orders);
        orders
    }

    /// Returns all active sell orders sorted by priority (best first)
    pub fn get_sell_orders(&self) -> Vec<Order> {
        let mut orders: Vec<Order> = Vec::new();
        for (_p, q) in self.asks.iter() {
            for o in q.iter() {
                if !self.canceled_orders.contains(&o.id) {
                    orders.push(o.clone());
                }
            }
        }
        orders.sort_by(compare_sell_orders);
        orders
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::{Order, OrderType};

    #[test]
    fn test_add_and_peek_orders() {
        let mut book = OrderBook::new();

        let buy1 = Order::new(1, Side::Buy, OrderType::Limit, 1000, 100, 1);
        let buy2 = Order::new(2, Side::Buy, OrderType::Limit, 1060, 100, 2);
        let sell1 = Order::new(3, Side::Sell, OrderType::Limit, 1100, 100, 3);

        book.add_order(buy1);
        book.add_order(buy2);
        book.add_order(sell1);

        let best_buy = book.peek_best_buy().unwrap();
        let best_sell = book.peek_best_sell().unwrap();

        assert_eq!(best_buy.id, 2);
        assert_eq!(best_sell.id, 3);
    }

    #[test]
    fn test_pop_best_orders() {
        let mut book = OrderBook::new();

        let buy1 = Order::new(1, Side::Buy, OrderType::Limit, 1000, 100, 1);
        let buy2 = Order::new(2, Side::Buy, OrderType::Limit, 1060, 100, 2);
        let sell1 = Order::new(3, Side::Sell, OrderType::Limit, 1100, 100, 3);

        book.add_order(buy1);
        book.add_order(buy2);
        book.add_order(sell1);

        let best_buy = book.pop_best_buy().unwrap();
        let best_sell = book.pop_best_sell().unwrap();

        assert_eq!(best_buy.id, 2);
        assert_eq!(best_sell.id, 3);

        // Orders should be removed from the book
        assert!(book.peek_best_buy().is_some()); // buy1 still there
        assert_eq!(book.peek_best_buy().unwrap().id, 1);
        assert!(book.peek_best_sell().is_none());
    }

    #[test]
    fn test_order_retrieval() {
        let mut book = OrderBook::new();

        // Test that we can add and retrieve orders with get_* methods
        let buy1 = Order::new(1, Side::Buy, OrderType::Limit, 1000, 100, 2);
        let buy2 = Order::new(2, Side::Buy, OrderType::Limit, 1001, 100, 5);
        book.add_order(buy1);
        book.add_order(buy2);

        let buy_orders = book.get_buy_orders();
        assert_eq!(buy_orders.len(), 2);
        assert_eq!(buy_orders[0].id, 2);

        let sell1 = Order::new(3, Side::Sell, OrderType::Limit, 1100, 100, 3);
        let sell2 = Order::new(4, Side::Sell, OrderType::Limit, 1100, 100, 2);
        book.add_order(sell1);
        book.add_order(sell2);

        let sell_orders = book.get_sell_orders();
        assert_eq!(sell_orders.len(), 2);
        assert_eq!(sell_orders[0].id, 4);
    }

    #[test]
    fn test_same_price_timestamp_priority() {
        let mut book = OrderBook::new();

        // Buy orders at same price - earlier timestamp should have priority
        let buy1 = Order::new(1, Side::Buy, OrderType::Limit, 1000, 100, 5);
        let buy2 = Order::new(2, Side::Buy, OrderType::Limit, 1000, 100, 2);

        book.add_order(buy1);
        book.add_order(buy2);

        let best = book.pop_best_buy().unwrap();
        assert_eq!(best.id, 2); // Earlier timestamp should be popped first
    }

    #[test]
    fn test_multiple_price_levels() {
        let mut book = OrderBook::new();

        let buy1 = Order::new(1, Side::Buy, OrderType::Limit, 1000, 100, 1);
        let buy2 = Order::new(2, Side::Buy, OrderType::Limit, 1020, 100, 2);
        let buy3 = Order::new(3, Side::Buy, OrderType::Limit, 1040, 100, 3);

        book.add_order(buy1);
        book.add_order(buy2);
        book.add_order(buy3);

        let mut orders = Vec::new();
        while let Some(order) = book.pop_best_buy() {
            orders.push(order.id);
        }

        // Should be popped in descending price order: 1040, 1020, 1000
        assert_eq!(orders, vec![3, 2, 1]);
    }
}

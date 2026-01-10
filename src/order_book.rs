// the fun stuff: Order Book implementation
//
//

use crate::order::{Order, OrderId, Side, compare_buy_orders, compare_sell_orders};
use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    collections::{HashMap, HashSet},
};

#[derive(Clone, Debug)]
struct OrderWrapper {
    order: Order,
    is_buy: bool,
}

// the following trait impls are essential for the BinaryHeap ds (which requires Ord trait)
impl PartialEq for OrderWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.order.id == other.order.id
    }
}

impl Eq for OrderWrapper {}

impl PartialOrd for OrderWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_buy {
            compare_buy_orders(&self.order, &other.order)
        } else {
            compare_sell_orders(&self.order, &other.order)
        }
    }
}

pub struct OrderBook {
    buy_orders: BinaryHeap<OrderWrapper>,
    sell_orders: BinaryHeap<OrderWrapper>,
    orders_map: HashMap<OrderId, Order>,
    canceled_orders: HashSet<OrderId>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            buy_orders: BinaryHeap::new(),
            sell_orders: BinaryHeap::new(),
            orders_map: HashMap::<OrderId, Order>::new(),
            canceled_orders: HashSet::<OrderId>::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let order_id = order.id;
        let side = order.side;

        // to lookup map
        self.orders_map.insert(order_id, order.clone());

        let wrapper = OrderWrapper {
            order,
            is_buy: side == Side::Buy,
        };

        // to the appropriate side
        match side {
            Side::Buy => self.buy_orders.push(wrapper),
            Side::Sell => self.sell_orders.push(wrapper),
        }
    }

    // the following peek functions return the best buy/sell without removing them from the heap
    pub fn peek_best_buy(&mut self) -> Option<Order> {
        while let Some(wrapper) = self.buy_orders.peek() {
            if self.canceled_orders.contains(&wrapper.order.id) {
                self.buy_orders.pop();
            } else {
                return Some(wrapper.order.clone());
            }
        }
        None
    }
    pub fn peek_best_sell(&mut self) -> Option<Order> {
        while let Some(wrapper) = self.sell_orders.peek() {
            if self.canceled_orders.contains(&wrapper.order.id) {
                self.sell_orders.pop();
            } else {
                return Some(wrapper.order.clone());
            }
        }
        None
    }

    pub fn pop_best_buy(&mut self) -> Option<Order> {
        loop {
            match self.buy_orders.pop() {
                Some(wrapper) => {
                    if !self.canceled_orders.contains(&wrapper.order.id) {
                        self.orders_map.remove(&wrapper.order.id);
                        return Some(wrapper.order);
                    }
                }

                None => return None,
            }
        }
    }
    pub fn pop_best_sell(&mut self) -> Option<Order> {
        loop {
            match self.sell_orders.pop() {
                Some(wrapper) => {
                    if !self.canceled_orders.contains(&wrapper.order.id) {
                        self.orders_map.remove(&wrapper.order.id);
                        return Some(wrapper.order);
                    }
                }

                None => return None,
            }
        }
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> bool {
        if self.orders_map.contains_key(&order_id) {
            self.canceled_orders.insert(order_id);
            self.orders_map.remove(&order_id);
            true
        } else {
            false
        }
    }

    /// Returns all active buy orders sorted by priority (best first)
    pub fn get_buy_orders(&self) -> Vec<Order> {
        let mut orders: Vec<Order> = self
            .orders_map
            .values()
            .filter(|o| o.side == Side::Buy && !self.canceled_orders.contains(&o.id))
            .cloned()
            .collect();
        orders.sort_by(compare_buy_orders);
        orders
    }

    /// Returns all active sell orders sorted by priority (best first)
    pub fn get_sell_orders(&self) -> Vec<Order> {
        let mut orders: Vec<Order> = self
            .orders_map
            .values()
            .filter(|o| o.side == Side::Sell && !self.canceled_orders.contains(&o.id))
            .cloned()
            .collect();
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
}

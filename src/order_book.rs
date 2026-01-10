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

    pub fn add_order(&mut self, order: Order) {}
}

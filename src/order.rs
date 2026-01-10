use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

// rudimentary types
pub type OrderId = u64;

pub type Price = u64;

pub type Quantity = u64;

pub type Timestamp = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: Timestamp,
}

// order's methods

impl Order {
    pub fn new(
        id: OrderId,
        side: Side,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
        timestamp: Timestamp,
    ) -> Self {
        Order {
            id,
            side,
            order_type,
            price,
            quantity,
            timestamp,
        }
    }

    pub fn can_match(&self, other: &Order) -> bool {
        if self.side == other.side {
            return false;
        }

        match (self.order_type, other.order_type) {
            (OrderType::Limit, OrderType::Limit) => {
                if self.side == Side::Buy {
                    self.price >= other.price // 
                } else {
                    self.price <= other.price
                }
            }

            _ => true, // market type orders always match with the best avail order (of opposite
                       // col. obviously)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    pub buy_order_id: OrderId,
    pub sell_order_id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
}

impl Trade {
    pub fn new(
        buy_order_id: OrderId,
        sell_order_id: OrderId,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Trade {
            buy_order_id,
            sell_order_id,
            price,
            quantity,
        }
    }
}

pub fn compare_buy_orders(a: &Order, b: &Order) -> Ordering {
    match a.price.cmp(&b.price) {
        Ordering::Greater => Ordering::Greater,
        Ordering::Less => Ordering::Less,
        Ordering::Equal => b.timestamp.cmp(&a.timestamp), // if same price, we move to timestamp
                                                          // comparison
    }
}

pub fn compare_sell_orders(a: &Order, b: &Order) -> Ordering {
    match a.price.cmp(&b.price) {
        Ordering::Less => Ordering::Greater, // reverse of the above (since lower sell prices are
        // given priority)
        Ordering::Greater => Ordering::Less,
        Ordering::Equal => b.timestamp.cmp(&a.timestamp), // if same price, we move to timestamp
                                                          // comparison
    }
}

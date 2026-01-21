use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt};

pub type Quantity = u64;
pub type Price = u64;
pub type Timestamp = u64;
pub type OrderId = String;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub quantity: Quantity,
    pub price: Price,
    pub timestamp: Timestamp,
    pub side: Side,
    pub order_type: OrderType,
}

impl Order {
    pub fn new(
        id: OrderId,
        side: Side,
        order_type: OrderType,
        quantity: Quantity,
        price: Price,
        timestamp: Timestamp,
    ) -> Self {
        Order {
            id: id,
            quantity: quantity,
            price: price,
            side: side,
            order_type: order_type,
            timestamp: timestamp,
        }
    }

    pub fn can_match(&self, other: &Order) -> bool {
        if self.side == other.side {
            return false;
        }

        match (self.order_type, other.order_type) {
            (OrderType::Limit, OrderType::Limit) => {
                if self.side == Side::Buy {
                    self.price >= other.price // check if the buy order's price is greater than the
                // existing sell order's price
                } else {
                    self.price <= other.price // check if the sell order's price is less than the
                    // existing buy order's price
                }
            }

            _ => true, // market type orders always match with the best avail order (of opposite
                       // col. obviously)
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.side, &self.order_type) {
            (Side::Buy, OrderType::Limit) => write!(
                f,
                "\nID: {}\nSide: Buy\nOrder Type: Limit\nQuantity: {}\nPrice: {}\nTimestamp: {}\n",
                self.id, self.quantity, self.price, self.timestamp
            ),
            (Side::Buy, OrderType::Market) => write!(
                f,
                "\nID: {}\nSide: Buy\nOrder Type: Market\nQuantity: {}\nPrice: {}\nTimestamp: {}\n",
                self.id, self.quantity, self.price, self.timestamp
            ),
            (Side::Sell, OrderType::Market) => write!(
                f,
                "\nID: {}\nSide: Sell\nOrder Type: Market\nQuantity: {}\nPrice: {}\nTimestamp: {}\n",
                self.id, self.quantity, self.price, self.timestamp
            ),
            (Side::Sell, OrderType::Limit) => write!(
                f,
                "\nID: {}\nSide: Sell\nOrder Type: Limit\nQuantity: {}\nPrice: {}\nTimestamp: {}\n",
                self.id, self.quantity, self.price, self.timestamp
            ),
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

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nTrade\nBuy ID: {}\nSell ID: {}\nPrice: {}\nQuantity: {}\n",
            self.buy_order_id, self.sell_order_id, self.price, self.quantity
        )
    }
}

pub fn _compare_buy_orders(o1: &Order, o2: &Order) -> Ordering {
    match o1.price.cmp(&o2.price) {
        Ordering::Less => Ordering::Greater,
        Ordering::Greater => Ordering::Less,
        Ordering::Equal => o1.timestamp.cmp(&o2.timestamp),
    }
}

pub fn _compare_sell_orders(o1: &Order, o2: &Order) -> Ordering {
    match o1.price.cmp(&o2.price) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => o1.timestamp.cmp(&o2.timestamp),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_order_display_format() {
        let o1 = Order::new("1".to_string(), Side::Buy, OrderType::Limit, 2000, 10, 1);
        println!("{}", o1);
    }

    #[test]
    fn test_trade_display_format() {
        let t1 = Trade::new("1".to_string(), "1".to_string(), 10, 2000);
        println!("{}", t1);
    }

    #[test]
    fn test_buy_orders_ordering() {
        let o1 = Order::new("1".to_string(), Side::Buy, OrderType::Limit, 2000, 10, 1);
        let o2 = Order::new("2".to_string(), Side::Buy, OrderType::Limit, 2000, 20, 2);

        assert_eq!(_compare_buy_orders(&o1, &o2), Ordering::Greater);

        let ot1 = Order::new("1".to_string(), Side::Buy, OrderType::Limit, 2000, 10, 1);
        let ot2 = Order::new("2".to_string(), Side::Buy, OrderType::Limit, 2000, 10, 2);

        assert_eq!(_compare_buy_orders(&ot1, &ot2), Ordering::Less);
    }

    #[test]
    fn test_sell_orders_ordering() {
        let o1 = Order::new("1".to_string(), Side::Sell, OrderType::Limit, 2000, 10, 1);
        let o2 = Order::new("2".to_string(), Side::Sell, OrderType::Limit, 2000, 20, 2);

        assert_eq!(_compare_sell_orders(&o1, &o2), Ordering::Less);

        let ot1 = Order::new("1".to_string(), Side::Sell, OrderType::Limit, 2000, 10, 1);
        let ot2 = Order::new("2".to_string(), Side::Sell, OrderType::Limit, 2000, 10, 2);

        assert_eq!(_compare_sell_orders(&ot1, &ot2), Ordering::Less);
    }
}

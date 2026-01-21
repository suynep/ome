use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

use crate::order::{Order, OrderId, OrderType, Price, Quantity, Side};

pub struct OrderBook {
    pub bids: BTreeMap<Price, Vec<Order>>,
    pub asks: BTreeMap<Price, Vec<Order>>,
    pub order_map: HashMap<OrderId, (Quantity, Price, Side)>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::<Price, Vec<Order>>::new(),
            asks: BTreeMap::<Price, Vec<Order>>::new(),
            order_map: HashMap::new(), // keep track of ALL the orders in the book, regardless of
                                       // side
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let side = order.side;

        match side {
            Side::Buy => {
                let queue = self.bids.entry(order.price).or_insert_with(Vec::new);

                let pos = queue
                    .iter()
                    .position(|ele| ele.timestamp > order.timestamp)
                    .unwrap_or(queue.len()); // iterate over the vector to find the first timestamp
                // greater than the current timestamp and return the position

                queue.insert(pos, order.clone());
            }

            Side::Sell => {
                let queue = self.asks.entry(order.price).or_insert_with(Vec::new);

                let pos = queue
                    .iter()
                    .position(|ele| ele.timestamp > order.timestamp)
                    .unwrap_or(queue.len()); // iterate over the vector to find the first timestamp
                // greater than the current timestamp and return the position

                queue.insert(pos, order.clone());
            }
        }

        // insert orders to the heap ONLY if they are of LIMIT type
        // if order.order_type != OrderType::Market {
        self.order_map
            .insert(order.id, (order.quantity, order.price, order.side));
        // }
    }

    pub fn peek_best_buy(&mut self) -> Option<Order> {
        loop {
            let (best_price, _) = match self.bids.last_key_value() {
                Some((p, q)) => (*p, q),
                None => return None,
            };

            if let Some(q) = self.bids.get_mut(&best_price) {
                if q.is_empty() {
                    self.bids.remove(&best_price);
                    continue;
                }
                if let Some(front) = q.first() {
                    return Some(front.clone());
                }
            }
        }
    }

    pub fn pop_best_buy(&mut self) -> Option<Order> {
        loop {
            let (best_price, _) = match self.bids.last_key_value() {
                Some((p, q)) => (*p, q),
                None => return None,
            };

            if let Some(q) = self.bids.get_mut(&best_price) {
                if q.is_empty() {
                    self.bids.remove(&best_price);
                    continue;
                }
                if let Some(_) = q.first() {
                    let front = q.remove(0);
                    return Some(front);
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
                if q.is_empty() {
                    self.asks.remove(&best_price);
                    continue;
                }
                if let Some(front) = q.first() {
                    return Some(front.clone());
                }
            }
        }
    }

    pub fn pop_best_sell(&mut self) -> Option<Order> {
        loop {
            let (best_price, _) = match self.asks.first_key_value() {
                Some((p, q)) => (*p, q),
                None => return None,
            };

            if let Some(q) = self.asks.get_mut(&best_price) {
                if q.is_empty() {
                    self.asks.remove(&best_price);
                    continue;
                }
                if let Some(_) = q.first() {
                    let front = q.remove(0);
                    return Some(front);
                }
            }
        }
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> bool {
        // we extract the side from the order_map
        if let Some(ord) = self.order_map.get(&order_id) {
            let side = ord.2; // side
            let price = ord.1; // price
            let removed = match side {
                Side::Buy => {
                    if let Some(q) = self.bids.get_mut(&price) {
                        if let Some(ind) = q.iter().position(|e| e.id == order_id) {
                            q.remove(ind);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                Side::Sell => {
                    if let Some(q) = self.asks.get_mut(&price) {
                        if let Some(ind) = q.iter().position(|e| e.id == order_id) {
                            q.remove(ind);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            };

            if removed {
                self.order_map.remove(&order_id);
            }

            removed
        } else {
            false
        }
    }

    pub fn get_buy_orders(&self) -> Vec<Order> {
        let mut buy_orders = Vec::<Order>::new();
        for (_, v) in self.bids.iter() {
            for bo in v {
                buy_orders.push(bo.clone());
            }
        }

        buy_orders
    }
    pub fn get_sell_orders(&self) -> Vec<Order> {
        let mut sell_orders = Vec::<Order>::new();
        for (_, v) in self.asks.iter() {
            for bo in v {
                sell_orders.push(bo.clone());
            }
        }

        sell_orders
    }
}

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let mut bfstr = String::from("");
        // let mut afstr = String::from("");
        let _ = write!(f, "\n");
        let _ = write!(f, "Buy:\n");
        for (price, queue) in self.bids.iter() {
            let _ = write!(f, "{} -> {:?}\n", price, queue);
        }

        let _ = write!(f, "\n");
        let _ = write!(f, "Sell:\n");
        for (price, queue) in self.asks.iter() {
            let _ = write!(f, "{} -> {:?}\n", price, queue);
        }

        write!(f, "\n")
        // write!(f, bfstr)
    }
}

impl Clone for OrderBook {
    fn clone(&self) -> Self {
        OrderBook {
            bids: self.bids.clone(),
            asks: self.asks.clone(),
            order_map: self.order_map.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_orderbook_display_format() {
        let mut ob = OrderBook::new();
        let o1 = Order::new(String::from("1"), Side::Buy, OrderType::Limit, 2000, 10, 1);
        let o2 = Order::new(String::from("2"), Side::Buy, OrderType::Limit, 2000, 200, 2);
        let o3 = Order::new(String::from("3"), Side::Buy, OrderType::Limit, 2000, 200, 1);
        ob.add_order(o1);
        ob.add_order(o2);
        ob.add_order(o3);

        println!("Hello, world");
        println!("{}", ob);
    }

    #[test]
    fn test_peek_best_buy() {
        let mut ob = OrderBook::new();
        let o1 = Order::new(String::from("1"), Side::Buy, OrderType::Limit, 2000, 10, 1);
        let o2 = Order::new(String::from("2"), Side::Buy, OrderType::Limit, 2000, 200, 2);
        let o3 = Order::new(String::from("3"), Side::Buy, OrderType::Limit, 2000, 200, 1);
        let o4 = Order::new(String::from("4"), Side::Buy, OrderType::Limit, 2000, 500, 1);
        ob.add_order(o4);
        ob.add_order(o1);
        ob.add_order(o2);
        ob.add_order(o3);
        ob.peek_best_buy();
    }

    #[test]
    fn test_pop_best_buy() {
        let mut ob = OrderBook::new();
        let o1 = Order::new(String::from("1"), Side::Buy, OrderType::Limit, 2000, 10, 1);
        let o2 = Order::new(String::from("2"), Side::Buy, OrderType::Limit, 2000, 200, 2);
        let o3 = Order::new(String::from("3"), Side::Buy, OrderType::Limit, 2000, 200, 1);
        let o4 = Order::new(String::from("4"), Side::Buy, OrderType::Limit, 2000, 500, 1);
        ob.add_order(o4);
        ob.add_order(o1);
        ob.add_order(o2);
        ob.add_order(o3);

        println!("{}", ob);

        let v = ob.pop_best_buy().unwrap();
        println!("{}", v);
        let v = ob.pop_best_buy().unwrap();
        println!("{}", v);
        let v = ob.pop_best_buy().unwrap();
        println!("{}", v);
        let v = ob.pop_best_buy().unwrap();
        println!("{}", v);

        println!("{}", ob);
    }

    #[test]
    fn test_pop_best_sell() {
        let mut ob = OrderBook::new();
        let o1 = Order::new(String::from("1"), Side::Sell, OrderType::Limit, 2000, 10, 1);
        let o2 = Order::new(
            String::from("2"),
            Side::Sell,
            OrderType::Limit,
            2000,
            200,
            2,
        );
        let o3 = Order::new(
            String::from("3"),
            Side::Sell,
            OrderType::Limit,
            2000,
            200,
            1,
        );
        let o4 = Order::new(
            String::from("4"),
            Side::Sell,
            OrderType::Limit,
            2000,
            500,
            1,
        );
        ob.add_order(o4);
        ob.add_order(o1);
        ob.add_order(o2);
        ob.add_order(o3);
        println!("{}", ob);

        let v = ob.pop_best_sell().unwrap();
        println!("{}", v);
        let v = ob.pop_best_sell().unwrap();
        println!("{}", v);
        let v = ob.pop_best_sell().unwrap();
        println!("{}", v);

        println!("{}", ob);
    }

    #[test]
    fn test_get_buy_orders() {
        let mut ob = OrderBook::new();
        let o1 = Order::new(String::from("1"), Side::Sell, OrderType::Limit, 2000, 10, 1);
        let o2 = Order::new(String::from("2"), Side::Buy, OrderType::Limit, 2000, 200, 2);
        let o3 = Order::new(String::from("3"), Side::Buy, OrderType::Limit, 2000, 200, 1);
        let o4 = Order::new(
            String::from("4"),
            Side::Sell,
            OrderType::Limit,
            2000,
            500,
            1,
        );
        ob.add_order(o4);
        ob.add_order(o1);
        ob.add_order(o2);
        ob.add_order(o3);
        println!("{}", ob);

        println!("{:?}", ob.get_buy_orders());
    }

    #[test]
    fn test_get_sell_orders() {
        let mut ob = OrderBook::new();
        let o1 = Order::new(String::from("1"), Side::Sell, OrderType::Limit, 2000, 10, 1);
        let o2 = Order::new(String::from("2"), Side::Buy, OrderType::Limit, 2000, 200, 2);
        let o3 = Order::new(String::from("3"), Side::Buy, OrderType::Limit, 2000, 200, 1);
        let o4 = Order::new(
            String::from("4"),
            Side::Sell,
            OrderType::Limit,
            2000,
            500,
            1,
        );
        ob.add_order(o4);
        ob.add_order(o1);
        ob.add_order(o2);
        ob.add_order(o3);
        println!("{}", ob);

        println!("{:?}", ob.get_sell_orders());
    }

    #[test]
    fn test_cancellation() {
        let mut ob = OrderBook::new();
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
        ob.add_order(o4);
        ob.add_order(o1);
        ob.add_order(o2);
        ob.add_order(o3);
        ob.add_order(o5);
        ob.add_order(o6);

        println!("{}", ob);
        println!("Order_Map: {:?}", ob.order_map);

        ob.cancel_order(String::from("6"));

        println!("{}", ob);
        println!("Order_Map: {:?}", ob.order_map);
    }
}

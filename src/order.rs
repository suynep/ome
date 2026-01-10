// rudimentary types
pub type OrderId = u64;

pub type Price = u64;

pub type Quantity = u64;

pub type Timestamp = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone)]
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

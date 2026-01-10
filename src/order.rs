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

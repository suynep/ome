use crate::order::{OrderId, Timestamp, Trade};
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
}

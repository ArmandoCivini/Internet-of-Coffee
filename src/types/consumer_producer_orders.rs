use std_semaphore::Semaphore;
use std::sync::RwLock;
use crate::types::state::State;
use crate::types::order_format::OrderFormat;

pub struct ConsumerProducerOrders {
    pub not_empty: Semaphore,
    pub not_full: Semaphore,
    pub orders: RwLock<Vec<OrderFormat>>,
    pub stop: RwLock<State>,
}
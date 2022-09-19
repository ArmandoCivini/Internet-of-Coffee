use crate::types::order_format::OrderFormat;
use crate::types::state::State;
use std::sync::RwLock;
use std_semaphore::Semaphore;

pub struct ConsumerProducerOrders {
    pub not_empty: Semaphore,
    pub not_full: Semaphore,
    pub orders: RwLock<Vec<OrderFormat>>,
    pub stop: RwLock<State>,
}

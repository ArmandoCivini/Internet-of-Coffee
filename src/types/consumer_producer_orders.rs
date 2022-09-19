use std_semaphore::Semaphore;
use std::sync::RwLock;
use crate::types::state::State;

pub struct ConsumerProducerOrders {
    pub not_empty: Semaphore,
    pub not_full: Semaphore,
    pub orders: RwLock<Vec<i32>>,
    pub stop: RwLock<State>,
}
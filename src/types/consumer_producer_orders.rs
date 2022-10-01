use crate::sync::RwLock;
use crate::types::order_format::OrderFormat;
use crate::types::state::State;
use std_semaphore::Semaphore;

///Estructuras necesarias para un buffer con el patron productor-consumidor.
pub struct ConsumerProducerOrders {
    pub not_empty: Semaphore,
    pub not_full: Semaphore,
    pub orders: RwLock<Vec<OrderFormat>>,
    pub stop: RwLock<State>,
}

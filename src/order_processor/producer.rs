use crate::types::state::State;
use std::sync::{Arc, RwLock};
use std_semaphore::Semaphore;

pub fn producer(
    orders_buffer_lock: Arc<RwLock<Vec<i32>>>,
    orders_not_empty: Arc<Semaphore>,
    orders_not_full: Arc<Semaphore>,
    stop: Arc<RwLock<State>>,
) {
    let mut i = 0;
    while i < 50 {
        orders_not_full.acquire();
        {
            let mut buffer = orders_buffer_lock.write().unwrap();
            buffer.push(i);
        }
        orders_not_empty.release();
        i += 1;
    }
    //orders_not_full.acquire();
    let mut stop_write = stop.write().unwrap();
    *stop_write = State::FinishedReading;
    println!("finished producer");
    for _i in 0..100 {
        orders_not_empty.release();
    }
}

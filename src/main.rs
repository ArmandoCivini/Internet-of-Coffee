use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std_semaphore::Semaphore;
mod types;
use types::state::State;
mod coffee_maker;
use coffee_maker::consumer;
mod order_processor;
use order_processor::producer;

fn main() {
    let dispensers_number = 10;
    let orders_buffer_size = 20;
    let stop = State::Reading;
    let orders_not_empty = Arc::new(Semaphore::new(0));
    let orders_not_full = Arc::new(Semaphore::new(orders_buffer_size));
    let orders_buffer: Vec<i32> = Vec::new();
    let orders_buffer_lock = Arc::new(RwLock::new(orders_buffer));
    let stop_lock = Arc::new(RwLock::new(stop));
    let dispensers_threads: Vec<JoinHandle<()>> = (0..dispensers_number)
        .map(|_| {
            let lock_clone = orders_buffer_lock.clone();
            let orders_not_empty_clone = orders_not_empty.clone();
            let orders_not_full_clone = orders_not_full.clone();
            let stop_lock_clone = stop_lock.clone();
            thread::spawn(move || {
                consumer::consumer(
                    lock_clone,
                    orders_not_empty_clone,
                    orders_not_full_clone,
                    stop_lock_clone,
                )
            })
        })
        .collect();
    producer::producer(
        orders_buffer_lock,
        orders_not_empty,
        orders_not_full,
        stop_lock,
    );
    dispensers_threads
        .into_iter()
        .flat_map(|x| x.join())
        .for_each(drop)
}

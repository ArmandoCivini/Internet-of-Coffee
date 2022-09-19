use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std_semaphore::Semaphore;
mod types;
use types::state::State;
use types::consumer_producer_orders::ConsumerProducerOrders;
mod coffee_maker;
use coffee_maker::consumer;
mod order_processor;
use order_processor::producer;

fn main() {
    let dispensers_number = 10;
    let orders_buffer_size = 20;

    let consumer_producer_orders = ConsumerProducerOrders {
        not_empty: Semaphore::new(0),
        not_full: Semaphore::new(orders_buffer_size),
        orders: RwLock::new(Vec::new()),
        stop: RwLock::new(State::Reading),
    };

    let consumer_producer_orders_ref = Arc::new(consumer_producer_orders);

    let dispensers_threads: Vec<JoinHandle<()>> = (0..dispensers_number)
        .map(|_| {
            let consumer_producer_orders_clone = consumer_producer_orders_ref.clone();
            thread::spawn(move || {
                consumer::consumer(
                    consumer_producer_orders_clone
                )
            })
        })
        .collect();
    producer::producer(
        consumer_producer_orders_ref
    );
    dispensers_threads
        .into_iter()
        .flat_map(|x| x.join())
        .for_each(drop)
}

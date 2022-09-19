use crate::types::consumer_producer_orders::ConsumerProducerOrders;
use crate::types::ingridients::Ingridients;
use crate::types::order_format::OrderFormat;
use crate::types::state::State;
use std::sync::{Arc, Condvar, Mutex};

fn dispenser(order: OrderFormat, ingridients_pair: &Arc<(Mutex<Ingridients>, Condvar)>) {
    let (lock, cvar) = &**ingridients_pair;
    println!("{}", order);
    let mut ingridients = lock.lock().unwrap();
    println!("{}", ingridients);
    ingridients.e -= 1;
}

pub fn consumer(
    order_resources: Arc<ConsumerProducerOrders>,
    ingridients_pair: Arc<(Mutex<Ingridients>, Condvar)>,
) {
    let mut cond: State;
    {
        let stop_read = order_resources.stop.read().unwrap();
        cond = *stop_read;
    }
    while !matches!(cond, State::FinishedProcessing) {
        order_resources.not_empty.acquire();
        {
            let mut buffer = order_resources.orders.write().unwrap();
            if buffer.len() == 0 {
                if matches!(cond, State::FinishedReading) {
                    println!("alerting");
                    let mut stop_write = order_resources.stop.write().unwrap();
                    *stop_write = State::FinishedProcessing;
                }
                {
                    let stop_read = order_resources.stop.read().unwrap();
                    cond = *stop_read;
                }
                continue;
            }
            dispenser(buffer.remove(0), &ingridients_pair);
            order_resources.not_full.release();
        }
        {
            let stop_read = order_resources.stop.read().unwrap();
            cond = *stop_read;
        }
    }
    println!("finished consumer");
}

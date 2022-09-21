use crate::types::consumer_producer_orders::ConsumerProducerOrders;
use crate::types::ingridients::Ingridients;
use crate::types::order_format::OrderFormat;
use crate::types::state::State;
use std::sync::{Arc, Condvar, Mutex};

fn dispenser(order: OrderFormat, ingridients_pair: &Arc<(Mutex<Ingridients>, Condvar)>) {
    let (lock, cvar) = &**ingridients_pair;
    println!("{}", order);
    let mut coffee_missing = order.coffee;
    let mut milk_missing = order.foam;
    while coffee_missing > 0 || milk_missing > 0 {
        let mut ingridient_guard = cvar
            .wait_while(lock.lock().unwrap(), |ingridients| {
                println!("[waiter] checking condition {}", *ingridients);
                if coffee_missing > 0 && ingridients.c > 0 {
                    return false;
                }
                if milk_missing > 0 && ingridients.e > 0 {
                    return false;
                }
                true
            })
            .unwrap();
        if ingridient_guard.c > coffee_missing {
            ingridient_guard.c -= coffee_missing;
            coffee_missing = 0;
        } else {
            coffee_missing -= ingridient_guard.c;
            ingridient_guard.c = 0;
        }
        if ingridient_guard.e > milk_missing {
            ingridient_guard.e -= milk_missing;
            milk_missing = 0;
        } else {
            milk_missing -= ingridient_guard.e;
            ingridient_guard.e = 0;
        }
        cvar.notify_all();
    }
    println!("finished fetching ingridients");
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

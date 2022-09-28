use crate::types::consumer_producer_orders::ConsumerProducerOrders;
use crate::types::ingridients::Ingridients;
use crate::types::order_format::OrderFormat;
use crate::types::state::State;
use crate::types::stats::Stats;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::{thread, time::Duration};

fn ingridient_sleep(ingridient: i32) {
    thread::sleep(Duration::from_millis((ingridient as u64) * 100));
}
fn grab_ingridients(
    lock: &Mutex<Ingridients>,
    cvar: &Condvar,
    coffee_missing: &mut i32,
    milk_missing: &mut i32,
) {
    {
        let mut ingridient_guard = cvar
            .wait_while(lock.lock().expect("no se pudo lockear los ingredientes"), |ingridients| {
                println!("{}", *ingridients);
                if *coffee_missing > 0 && ingridients.c > 0 {
                    return false;
                }
                if *milk_missing > 0 && ingridients.e > 0 {
                    return false;
                }
                true
            })
            .expect("error en la condvar de los ingredientes");
        if ingridient_guard.c > *coffee_missing {
            ingridient_guard.c -= *coffee_missing;
            *coffee_missing = 0;
        } else {
            *coffee_missing -= ingridient_guard.c;
            ingridient_guard.c = 0;
        }
        if ingridient_guard.e > *milk_missing {
            ingridient_guard.e -= *milk_missing;
            *milk_missing = 0;
        } else {
            *milk_missing -= ingridient_guard.e;
            ingridient_guard.e = 0;
        }
    }
    cvar.notify_all();
}
fn dispenser(order: &OrderFormat, ingridients_pair: &Arc<(Mutex<Ingridients>, Condvar)>) {
    let (lock, cvar) = &**ingridients_pair;
    println!("preparando orden: {{{}}}", order);
    let mut coffee_missing = order.coffee;
    let mut milk_missing = order.foam;
    while coffee_missing > 0 || milk_missing > 0 {
        grab_ingridients(lock, cvar, &mut coffee_missing, &mut milk_missing);
    }
    println!("se termino de recojer ingredientes");

    //tiempo de preparacion
    ingridient_sleep(order.coffee);
    ingridient_sleep(order.foam);
    ingridient_sleep(order.hot_water);
}

fn register_order(order: &OrderFormat, stats_lock: &Arc<RwLock<Stats>>) {
    let mut stats = stats_lock.write().expect("no se pudo escribir en los stats");
    stats.c_consumed += order.coffee;
    stats.e_consumed += order.foam;
    stats.water_consumed += order.hot_water;
    stats.coffee_consumed += 1;
}

pub fn consumer(
    order_resources: Arc<ConsumerProducerOrders>,
    ingridients_pair: Arc<(Mutex<Ingridients>, Condvar)>,
    stats: Arc<RwLock<Stats>>,
) {
    let mut cond: State;
    let mut order: OrderFormat;
    {
        let stop_read = order_resources.stop.read().expect("no se pudo leer en el stop");
        cond = *stop_read;
    }
    while !matches!(cond, State::FinishedProcessing) {
        order_resources.not_empty.acquire();
        {
            let mut buffer = order_resources.orders.write().expect("no se pudo escribir en el buffer de ordenes");
            if buffer.len() == 0 {
                if matches!(cond, State::FinishedReading) {
                    println!("alertando");
                    let mut stop_write = order_resources.stop.write().expect("no se pudo escribir en el stop");
                    *stop_write = State::FinishedProcessing;
                }
                {
                    let stop_read = order_resources.stop.read().expect("no se pudo leer en el stop");
                    cond = *stop_read;
                }
                continue;
            }
            order = buffer.remove(0);
            order_resources.not_full.release();
        }
        dispenser(&order, &ingridients_pair);
        register_order(&order, &stats);
        {
            let stop_read = order_resources.stop.read().expect("no se pudo leer en el stop");
            cond = *stop_read;
        }
    }
    println!("fin de consumidor");
}

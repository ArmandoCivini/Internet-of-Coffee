use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::{thread};
use std::thread::JoinHandle;
use std_semaphore::Semaphore;

mod types;
use types::consumer_producer_orders::ConsumerProducerOrders;
use types::ingridients::Ingridients;
use types::state::State;
use types::stats::Stats;

mod coffee_maker;
use coffee_maker::consumer;

mod order_processor;
use order_processor::producer;

mod ingridient_reloader;
use crate::ingridient_reloader::ingridient_reloader::ingridient_reloader;

mod display_stats;
use crate::display_stats::display_stats::display_stats;

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

    let ingridients = Ingridients {
        g: 100,
        c: 100,
        l: 100,
        e: 100,
    };
    let ingridients_pair = Arc::new((Mutex::new(ingridients), Condvar::new()));

    let stats = Stats {
        g_consumed: 0,
        c_consumed: 0,
        l_consumed: 0,
        e_consumed: 0,
        water_consumed: 0,
        coffee_consumed: 0,
    };
    let stats_ref = Arc::new(RwLock::new(stats));

    let dispensers_threads: Vec<JoinHandle<()>> = (0..dispensers_number)
        .map(|_| {
            let consumer_producer_orders_clone = consumer_producer_orders_ref.clone();
            let ingridients_pair_clone = ingridients_pair.clone();
            let stats_clone = stats_ref.clone();
            thread::spawn(move || {
                consumer::consumer(
                    consumer_producer_orders_clone,
                    ingridients_pair_clone,
                    stats_clone,
                )
            })
        })
        .collect();

    let end_of_orders = Arc::new(RwLock::new(false));
    let end_of_orders_clone = end_of_orders.clone();
    let ingridients_pair_clone = ingridients_pair.clone();
    let stats_clone = stats_ref.clone();
    let realoder_thread = thread::spawn(move || {
        ingridient_reloader(ingridients_pair_clone, end_of_orders_clone, stats_clone);
    });

    producer::producer(consumer_producer_orders_ref);

    let end_of_orders_clone_second = end_of_orders.clone();
    let stats_clone_second = stats_ref.clone();
    let stats_thread = thread::spawn(move || {
        display_stats(end_of_orders_clone_second, stats_clone_second);
    });

    dispensers_threads
        .into_iter()
        .flat_map(|x| x.join())
        .for_each(drop);
    {
        let mut stop_reloader = end_of_orders.write().expect("no se pudo escribir en stop");
        *stop_reloader = true;
    }
    let (lock, cvar) = &*ingridients_pair;
    {
        //desbloquea la condvar del reloader
        let mut ingridientss = lock.lock().expect("no se pudo conseguir el mutex de ingredientes");
        ingridientss.c = 0;
    }
    cvar.notify_all();
    realoder_thread.join().expect("no se pudo joinear la thread del recargador");
    stats_thread.join().expect("no se pudo joinear la thread de estadisticas");
}

use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::JoinHandle;
use std::{thread, time::Duration};
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

fn reload(ingridients_mutex: &Mutex<Ingridients>, reload_coffee: bool) {
    thread::sleep(Duration::from_millis(300));
    let mut ingridients = ingridients_mutex.lock().unwrap();
    if reload_coffee {
        //10 units of raw material can be converted to 100 units of product
        ingridients.c = 100;
        ingridients.g -= 10;
    } else {
        ingridients.e = 100;
        ingridients.l -= 10;
    }
    if ingridients.g == 0 {
        //if raw runs out, it is replentish with no cost
        ingridients.g = 100;
    }
    if ingridients.l == 0 {
        ingridients.l = 100;
    }
}

fn update_stats(stats_lock: &Arc<RwLock<Stats>>, reload_coffee: bool) {
    let mut stats = stats_lock.write().unwrap();
    if reload_coffee {
        stats.g_consumed += 10;
    } else {
        stats.l_consumed += 10;
    }
}

fn wait_missing_ingridients (lock: &Mutex<Ingridients>, cvar: &Condvar) -> bool {
    let ingridient_guard = cvar
        .wait_while(lock.lock().unwrap(), |ingridients| {
            ingridients.c > 0 && ingridients.e > 0
        })
        .unwrap();
    if ingridient_guard.c == 0 {
        println!("Reloading coffee");
        return true
    } else {
        println!("Reloading foam");
        return false
    }
}

fn ingridient_reloader(
    ingridients_pair: Arc<(Mutex<Ingridients>, Condvar)>,
    end_of_orders: Arc<RwLock<bool>>,
    stats: Arc<RwLock<Stats>>
) {
    let (lock, cvar) = &*ingridients_pair;
    let mut reload_coffee: bool;
    let mut cond: bool;
    {
        let stop_read = end_of_orders.read().unwrap();
        cond = *stop_read;
    }
    reload_coffee = wait_missing_ingridients(&lock, &cvar);
    cvar.notify_all();
    while !cond {
        reload(lock, reload_coffee);
        println!("Finished reloading");
        cvar.notify_all();
        update_stats(&stats, reload_coffee);
        reload_coffee = wait_missing_ingridients(&lock, &cvar);
        cvar.notify_all();
        {
            let stop_read = end_of_orders.read().unwrap();
            cond = *stop_read;
        }
    }
    println!("Shuting down reloader");
}

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
                    stats_clone
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

    dispensers_threads
        .into_iter()
        .flat_map(|x| x.join())
        .for_each(drop);
    {
        let mut stop_reloader = end_of_orders.write().unwrap();
        *stop_reloader = true;
    }
    let (lock, cvar) = &*ingridients_pair;
    {
        //unlocks the condvar on realoder
        let mut ingridientss = lock.lock().unwrap();
        ingridientss.c = 0;
    }
    cvar.notify_all();
    realoder_thread.join().unwrap();

    println!("{}", stats_ref.read().unwrap());
}

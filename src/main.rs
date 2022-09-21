use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std_semaphore::Semaphore;
mod types;
use types::consumer_producer_orders::ConsumerProducerOrders;
use types::ingridients::Ingridients;
use types::state::State;
mod coffee_maker;
use coffee_maker::consumer;
mod order_processor;
use order_processor::producer;

fn reload(ingridients_mutex: &Mutex<Ingridients>, reload_coffee: bool) {
    //TODO: add sleep
    let mut ingridients = ingridients_mutex.lock().unwrap();
    if reload_coffee {
        ingridients.c = 100;
        ingridients.g -= 10;
    } else {
        ingridients.e = 100;
        ingridients.l -= 10;
    }
}

fn ingridient_reloader(
    ingridients_pair: Arc<(Mutex<Ingridients>, Condvar)>,
    end_of_orders: Arc<RwLock<bool>>,
) {
    let (lock, cvar) = &*ingridients_pair;
    let mut reload_coffee: bool;
    let mut cond: bool;
    {
        let stop_read = end_of_orders.read().unwrap();
        cond = *stop_read;
    }
    while !cond {
        {
            let ingridient_guard = cvar
                .wait_while(lock.lock().unwrap(), |ingridients| {
                    ingridients.c > 0 && ingridients.e > 0
                })
                .unwrap();
            if ingridient_guard.c == 0 {
                reload_coffee = true;
            } else {
                reload_coffee = false;
            }
            cvar.notify_all();
        }
        reload(lock, reload_coffee);
        cvar.notify_all();
        {
            let stop_read = end_of_orders.read().unwrap();
            cond = *stop_read;
        }
    }
    println!("END");
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

    let dispensers_threads: Vec<JoinHandle<()>> = (0..dispensers_number)
        .map(|_| {
            let consumer_producer_orders_clone = consumer_producer_orders_ref.clone();
            let ingridients_pair_clone = ingridients_pair.clone();
            thread::spawn(move || {
                consumer::consumer(consumer_producer_orders_clone, ingridients_pair_clone)
            })
        })
        .collect();

    let end_of_orders = Arc::new(RwLock::new(false));
    let end_of_orders_clone = end_of_orders.clone();
    let ingridients_pair_clone = ingridients_pair.clone();
    let realoder_thread = thread::spawn(move || {
        ingridient_reloader(ingridients_pair_clone, end_of_orders_clone);
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
        let mut ingridientss = lock.lock().unwrap();
        ingridientss.c = 0;
    }
    cvar.notify_all();
    realoder_thread.join().unwrap();
}

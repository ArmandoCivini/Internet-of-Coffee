use std::{thread};
use std::thread::JoinHandle;
use std_semaphore::Semaphore;
use std::sync::{Arc, RwLock};

#[derive(Copy, Clone)]
enum State {
    Reading,
    FinishedReading,
    FinishedProcessing,
}

fn consumer(
    orders_buffer_lock: Arc<RwLock<Vec<i32>>>, 
    orders_not_empty: Arc<Semaphore>, 
    orders_not_full: Arc<Semaphore>,
    stop: Arc<RwLock<State>>) {
    let mut cond:State;
    {
        let stop_read = stop.read().unwrap();
        cond = *stop_read;
    }
    while !matches!(cond, State::FinishedProcessing) {
        orders_not_empty.acquire();
        {
            let mut buffer = orders_buffer_lock.write().unwrap();
            if buffer.len() == 0 {
                if matches!(cond, State::FinishedReading) {
                    println!("alerting");
                    let mut stop_write = stop.write().unwrap();
                    *stop_write = State::FinishedProcessing;
                }
                {
                    let stop_read = stop.read().unwrap();
                    cond = *stop_read;
                }
                continue;
            }
            println!("{}", buffer.remove(0));
            orders_not_full.release();
        }
        //thread::sleep(std::time::Duration::from_millis(500));
        {
            let stop_read = stop.read().unwrap();
            cond = *stop_read;
        }
    }
    //orders_not_full.release();
    println!("finished consumer");
}

fn producer(
    orders_buffer_lock: Arc<RwLock<Vec<i32>>>, 
    orders_not_empty: Arc<Semaphore>, 
    orders_not_full: Arc<Semaphore>,
    stop: Arc<RwLock<State>>) {
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
            thread::spawn(move || consumer(
                lock_clone, orders_not_empty_clone, 
                orders_not_full_clone, stop_lock_clone))
        })
        .collect();
    producer(orders_buffer_lock, orders_not_empty, orders_not_full, stop_lock);
    dispensers_threads.into_iter()
    .flat_map(|x| x.join())
    .for_each(drop)
}

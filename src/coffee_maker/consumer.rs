use crate::types::state::State;
use std::sync::{Arc, RwLock};
use std_semaphore::Semaphore;

pub fn consumer(
    orders_buffer_lock: Arc<RwLock<Vec<i32>>>,
    orders_not_empty: Arc<Semaphore>,
    orders_not_full: Arc<Semaphore>,
    stop: Arc<RwLock<State>>,
) {
    let mut cond: State;
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

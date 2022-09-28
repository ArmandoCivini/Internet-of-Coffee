use std::sync::{Arc, RwLock};
use std::{thread, time::Duration};
use crate::types::stats::Stats;

pub fn display_stats(stop: Arc<RwLock<bool>>, stats: Arc<RwLock<Stats>>) {
    let mut cond: bool;
    {
        let stop_read = stop.read().unwrap();
        cond = *stop_read;
    }
    while !cond {
        thread::sleep(Duration::from_millis(3000));
        {
        println!("{}", stats.read().unwrap());
        }
        {
            let stop_read = stop.read().unwrap();
            cond = *stop_read;
        }
    }
}

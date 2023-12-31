#[cfg(not(loom))]
use crate::print_mod::print_mod;
use crate::sync::{sleep, Arc, RwLock};
use crate::types::stats::Stats;
use std::time::Duration;

///Cada cierto intervalo de tiempo imprime en pantalla las estadisticas
#[cfg(not(loom))]
pub fn display_stats(stop: Arc<RwLock<bool>>, stats: Arc<RwLock<Stats>>) {
    let mut cond: bool;
    {
        let stop_read = stop.read().expect("no se pudo leer el stop");
        cond = *stop_read;
    }
    while !cond {
        sleep(Duration::from_millis(3000));
        {
            print_mod(format!(
                "{}",
                stats.read().expect("no se pudo leer los stats")
            ));
        }
        {
            let stop_read = stop.read().expect("no se pudo leer el stop");
            cond = *stop_read;
        }
    }
}

#[cfg(loom)]
#[warn(unused_variables)]
pub fn display_stats(_stop: Arc<RwLock<bool>>, _stats: Arc<RwLock<Stats>>) {}

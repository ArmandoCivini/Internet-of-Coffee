use crate::print_mod::print_mod;
use crate::types::stats::Stats;
use std::sync::{Arc, RwLock};
use std::{thread, time::Duration};

///Cada cierto intervalo de tiempo imprime en pantalla las estadisticas
pub fn display_stats(stop: Arc<RwLock<bool>>, stats: Arc<RwLock<Stats>>) {
    let mut cond: bool;
    {
        let stop_read = stop.read().expect("no se pudo leer el stop");
        cond = *stop_read;
    }
    while !cond {
        thread::sleep(Duration::from_millis(3000));
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

use crate::types::ingridients::Ingridients;
use crate::types::stats::Stats;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::{thread, time::Duration};

fn reload(ingridients_mutex: &Mutex<Ingridients>, reload_coffee: bool) {
    thread::sleep(Duration::from_millis(300));
    let mut ingridients = ingridients_mutex.lock().unwrap();
    if reload_coffee {
        //10 unidades de crudo son 100 unidades de producto
        ingridients.c = 100;
        ingridients.g -= 10;
    } else {
        ingridients.e = 100;
        ingridients.l -= 10;
    }
    if ingridients.g == 0 {
        //si se acaba crudo se recarga sin espera
        ingridients.g = 100;
    } else if ingridients.g < 25 {
        println!("capacidad de ganos de cafe por debajo del 25%");
    }
    if ingridients.l == 0 {
        ingridients.l = 100;
    } else if ingridients.l < 25 {
        println!("capacidad de leche fria por debajo del 25%");
    }
    println!("fin de la recarga");
}

fn update_stats(stats_lock: &Arc<RwLock<Stats>>, reload_coffee: bool) {
    let mut stats = stats_lock.write().unwrap();
    if reload_coffee {
        stats.g_consumed += 10;
    } else {
        stats.l_consumed += 10;
    }
}

fn wait_missing_ingridients(lock: &Mutex<Ingridients>, cvar: &Condvar) -> bool {
    let ingridient_guard = cvar
        .wait_while(lock.lock().unwrap(), |ingridients| {
            ingridients.c > 0 && ingridients.e > 0
        })
        .unwrap();
    if ingridient_guard.c == 0 {
        println!("Recargando cafe");
        return true;
    } else {
        println!("Recargando leche");
        return false;
    }
}

pub fn ingridient_reloader(
    ingridients_pair: Arc<(Mutex<Ingridients>, Condvar)>,
    end_of_orders: Arc<RwLock<bool>>,
    stats: Arc<RwLock<Stats>>,
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
        println!("Fin de recarga");
        cvar.notify_all();
        update_stats(&stats, reload_coffee);
        reload_coffee = wait_missing_ingridients(&lock, &cvar);
        cvar.notify_all();
        {
            let stop_read = end_of_orders.read().unwrap();
            cond = *stop_read;
        }
    }
    println!("Apagando recargador");
}

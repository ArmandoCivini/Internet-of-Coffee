use crate::print_mod::print_mod;
use crate::sync::{sleep, Arc, Condvar, Mutex, RwLock};
use crate::types::ingridients::Ingridients;
use crate::types::stats::Stats;
use std::time::Duration;

///Espera un tiempo de recarga y luego repone los ingredientes faltantes.
fn reload(ingridients_mutex: &Mutex<Ingridients>, reload_coffee: bool) {
    sleep(Duration::from_millis(300));
    let mut ingridients = ingridients_mutex
        .lock()
        .expect("no se pudo conseguir el mutex de ingredientes");
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
    } else if ingridients.g < 91 {
        print_mod("capacidad de ganos de cafe por debajo del 91%".to_string());
    }
    if ingridients.l == 0 {
        ingridients.l = 100;
    } else if ingridients.l < 91 {
        print_mod("capacidad de leche fria por debajo del 91%".to_string());
    }
    print_mod("fin de la recarga".to_string());
}

///Actualiza las estadisticas despues de recargar ingredientes.
fn update_stats(stats_lock: &Arc<RwLock<Stats>>, reload_coffee: bool) {
    let mut stats = stats_lock
        .write()
        .expect("no se pudo escribir en los stats");
    if reload_coffee {
        stats.g_consumed += 10;
    } else {
        stats.l_consumed += 10;
    }
}

///Espera hasta que haya algun ingrediente faltante.
#[cfg(not(loom))]
fn wait_missing_ingridients(lock: &Mutex<Ingridients>, cvar: &Condvar) -> bool {
    let ingridient_guard = cvar
        .wait_while(
            lock.lock().expect("no se pudo lockear los ingredientes"),
            |ingridients| ingridients.c > 0 && ingridients.e > 0,
        )
        .expect("fallo en la condvar de ingredientes");
    if ingridient_guard.c == 0 {
        print_mod("Recargando cafe".to_string());
        true
    } else {
        print_mod("Recargando leche".to_string());
        false
    }
}

#[cfg(loom)]
fn wait_missing_ingridients(_lock: &Mutex<Ingridients>, _cvar: &Condvar) -> bool {
    true
}

///Recarga ingredientes
pub fn ingridient_reloader(
    ingridients_pair: Arc<(Mutex<Ingridients>, Condvar)>,
    end_of_orders: Arc<RwLock<bool>>,
    stats: Arc<RwLock<Stats>>,
) {
    let (lock, cvar) = &*ingridients_pair;
    let mut reload_coffee: bool;
    let mut cond: bool;
    reload_coffee = wait_missing_ingridients(lock, cvar);
    cvar.notify_all();
    {
        let stop_read = end_of_orders
            .read()
            .expect("no se pudo leer el stop de ordenes");
        cond = *stop_read;
    }
    while !cond {
        reload(lock, reload_coffee);
        print_mod("Fin de recarga".to_string());
        cvar.notify_all();
        update_stats(&stats, reload_coffee);
        reload_coffee = wait_missing_ingridients(lock, cvar);
        cvar.notify_all();
        {
            let stop_read = end_of_orders
                .read()
                .expect("no se pudo leer el stop de ordenes");
            cond = *stop_read;
        }
    }
    print_mod("Apagando recargador".to_string());
}

#[cfg(not(loom))]
#[cfg(test)]
mod tests {
    use crate::ingridient_reloader::{reload, update_stats};
    use crate::sync::{thread, Arc, Condvar, Mutex, RwLock};
    use crate::Ingridients;
    use crate::Stats;

    #[test]
    fn test_update_stats_coffee() {
        test_update_stats(true, 10, 0);
    }

    #[test]
    fn test_update_stats_mik() {
        test_update_stats(false, 0, 10);
    }

    fn test_update_stats(reload_coffee: bool, expected_g: i32, expected_l: i32) {
        let stats = Stats {
            g_consumed: 0,
            c_consumed: 0,
            l_consumed: 0,
            e_consumed: 0,
            water_consumed: 0,
            coffee_consumed: 0,
        };
        let stats_ref = Arc::new(RwLock::new(stats));

        let stats_lock = stats_ref.clone();

        update_stats(&stats_lock, reload_coffee);

        let stats_mut = stats_ref.read().expect("no se pudo leer stats");
        assert_eq!(0, stats_mut.c_consumed);
        assert_eq!(0, stats_mut.water_consumed);
        assert_eq!(0, stats_mut.e_consumed);
        assert_eq!(0, stats_mut.coffee_consumed);
        assert_eq!(expected_g, stats_mut.g_consumed);
        assert_eq!(expected_l, stats_mut.l_consumed);
    }

    #[test]
    fn test_reload_coffee() {
        let ingridients = Ingridients {
            g: 100,
            c: 0,
            l: 100,
            e: 100,
        };
        test_reload(true, ingridients);
    }

    #[test]
    fn test_reload_milk() {
        let ingridients = Ingridients {
            g: 100,
            c: 100,
            l: 100,
            e: 0,
        };
        test_reload(false, ingridients);
    }

    #[test]
    fn test_reload_coffee_both_empty() {
        let ingridients = Ingridients {
            g: 100,
            c: 0,
            l: 100,
            e: 0,
        };
        test_reload(true, ingridients);
    }

    #[test]
    fn test_reload_milk_both_empty() {
        let ingridients = Ingridients {
            g: 100,
            c: 0,
            l: 100,
            e: 0,
        };
        test_reload(false, ingridients);
    }

    fn test_reload(reload_coffee: bool, ingridients: Ingridients) {
        let g = ingridients.g;
        let c = ingridients.c;
        let l = ingridients.l;
        let e = ingridients.e;
        let ingridients_pair = Arc::new((Mutex::new(ingridients), Condvar::new()));
        let ingridients_pair_clone = ingridients_pair.clone();
        let reload_thread = thread::spawn(move || {
            let (lock, _) = &*ingridients_pair_clone;
            reload(lock, reload_coffee);
        });

        reload_thread.join().expect("no se pudo joinear el thread");
        let (lock, _) = &*ingridients_pair;

        let ingridients_ref = lock.lock().expect("no se pudo lockear el mutex");

        if reload_coffee {
            assert_eq!(100, ingridients_ref.c);
            assert_eq!(g - 10, ingridients_ref.g);
            assert_eq!(e, ingridients_ref.e);
            assert_eq!(l, ingridients_ref.l);
        } else {
            assert_eq!(c, ingridients_ref.c);
            assert_eq!(g, ingridients_ref.g);
            assert_eq!(100, ingridients_ref.e);
            assert_eq!(l - 10, ingridients_ref.l);
        }
    }

    #[test]
    fn test_raw_reload() {
        let ingridients = Ingridients {
            g: 10,
            c: 0,
            l: 0,
            e: 100,
        };

        let ingridients_pair = Arc::new((Mutex::new(ingridients), Condvar::new()));
        let ingridients_pair_clone = ingridients_pair.clone();
        let reload_thread = thread::spawn(move || {
            let (lock, _) = &*ingridients_pair_clone;
            reload(lock, true);
        });

        reload_thread.join().expect("no se pudo joinear el thread");
        let (lock, _) = &*ingridients_pair;

        let ingridients_ref = lock.lock().expect("no se pudo lockear el mutex");

        assert_eq!(100, ingridients_ref.c);
        assert_eq!(100, ingridients_ref.g);
        assert_eq!(100, ingridients_ref.e);
        assert_eq!(100, ingridients_ref.l);
    }
}

#[cfg(loom)]
#[cfg(test)]
mod tests {
    use crate::ingridient_reloader::{reload, update_stats};
    use crate::sync::{thread, Arc, Condvar, Mutex, RwLock};
    use crate::Ingridients;
    use crate::Stats;

    #[test]
    fn test_update_stats_coffee() {
        loom::model(|| {
            test_update_stats(true, 10, 0);
        });
    }

    fn test_update_stats(reload_coffee: bool, expected_g: i32, expected_l: i32) {
        let stats = Stats {
            g_consumed: 0,
            c_consumed: 0,
            l_consumed: 0,
            e_consumed: 0,
            water_consumed: 0,
            coffee_consumed: 0,
        };
        let stats_ref = Arc::new(RwLock::new(stats));

        let stats_lock = stats_ref.clone();

        update_stats(&stats_lock, reload_coffee);

        let stats_mut = stats_ref.read().expect("no se pudo leer stats");
        assert_eq!(0, stats_mut.c_consumed);
        assert_eq!(0, stats_mut.water_consumed);
        assert_eq!(0, stats_mut.e_consumed);
        assert_eq!(0, stats_mut.coffee_consumed);
        assert_eq!(expected_g, stats_mut.g_consumed);
        assert_eq!(expected_l, stats_mut.l_consumed);
    }

    #[test]
    fn test_reload_coffee() {
        loom::model(|| {
            let ingridients = Ingridients {
                g: 100,
                c: 0,
                l: 100,
                e: 100,
            };
            test_reload(true, ingridients);
        });
    }

    #[test]
    fn test_reload_milk_both_empty() {
        loom::model(|| {
            let ingridients = Ingridients {
                g: 100,
                c: 0,
                l: 100,
                e: 0,
            };
            test_reload(false, ingridients);
        });
    }

    fn test_reload(reload_coffee: bool, ingridients: Ingridients) {
        let g = ingridients.g;
        let c = ingridients.c;
        let l = ingridients.l;
        let e = ingridients.e;
        let ingridients_pair = Arc::new((Mutex::new(ingridients), Condvar::new()));
        let ingridients_pair_clone = ingridients_pair.clone();
        let reload_thread = thread::spawn(move || {
            let (lock, _) = &*ingridients_pair_clone;
            reload(lock, reload_coffee);
        });

        reload_thread.join().expect("no se pudo joinear el thread");
        let (lock, _) = &*ingridients_pair;

        let ingridients_ref = lock.lock().expect("no se pudo lockear el mutex");

        if reload_coffee {
            assert_eq!(100, ingridients_ref.c);
            assert_eq!(g - 10, ingridients_ref.g);
            assert_eq!(e, ingridients_ref.e);
            assert_eq!(l, ingridients_ref.l);
        } else {
            assert_eq!(c, ingridients_ref.c);
            assert_eq!(g, ingridients_ref.g);
            assert_eq!(100, ingridients_ref.e);
            assert_eq!(l - 10, ingridients_ref.l);
        }
    }

    #[test]
    fn test_raw_reload() {
        loom::model(move || {
            let ingridients = Ingridients {
                g: 10,
                c: 0,
                l: 0,
                e: 100,
            };

            let ingridients_pair = Arc::new((Mutex::new(ingridients), Condvar::new()));
            let ingridients_pair_clone_clone = ingridients_pair.clone();
            let reload_thread = thread::spawn(move || {
                let (lock, _) = &*ingridients_pair_clone_clone;
                reload(lock, true);
            });

            reload_thread.join().expect("no se pudo joinear el thread");
            let (lock, _) = &*ingridients_pair;

            let ingridients_ref = lock.lock().expect("no se pudo lockear el mutex");

            assert_eq!(100, ingridients_ref.c);
            assert_eq!(100, ingridients_ref.g);
            assert_eq!(100, ingridients_ref.e);
            assert_eq!(100, ingridients_ref.l);
        });
    }
}

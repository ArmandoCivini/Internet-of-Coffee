use crate::print_mod::print_mod;
use crate::sync::{sleep, Arc, Condvar, Mutex, RwLock};
use crate::types::consumer_producer_orders::ConsumerProducerOrders;
use crate::types::ingridients::Ingridients;
use crate::types::order_format::OrderFormat;
use crate::types::state::State;
use crate::types::stats::Stats;
use std::time::Duration;

///Cuento se espera para una cierta cantidad de ingredientes.
fn ingridient_sleep(ingridient: i32) {
    sleep(Duration::from_millis((ingridient as u64) * 100));
}

///Espera hasta que haya ingredientes para tomar.
/// Toma los ingredientes que nesecita si hay suficiente,
/// sino toma todo lo que haya.
#[cfg(not(loom))]
fn grab_ingridients(
    lock: &Mutex<Ingridients>,
    cvar: &Condvar,
    coffee_missing: &mut i32,
    milk_missing: &mut i32,
) {
    {
        let mut ingridient_guard = cvar
            .wait_while(
                lock.lock().expect("no se pudo lockear los ingredientes"),
                |ingridients: &mut Ingridients| {
                    print_mod(format!("{}", *ingridients));
                    if *coffee_missing > 0 && ingridients.c > 0 {
                        return false;
                    }
                    if *milk_missing > 0 && ingridients.e > 0 {
                        return false;
                    }
                    true
                },
            )
            .expect("error en la condvar de los ingredientes");
        if ingridient_guard.c > *coffee_missing {
            ingridient_guard.c -= *coffee_missing;
            *coffee_missing = 0;
        } else {
            *coffee_missing -= ingridient_guard.c;
            ingridient_guard.c = 0;
        }
        if ingridient_guard.e > *milk_missing {
            ingridient_guard.e -= *milk_missing;
            *milk_missing = 0;
        } else {
            *milk_missing -= ingridient_guard.e;
            ingridient_guard.e = 0;
        }
    }
    cvar.notify_all();
}

#[cfg(loom)]
fn grab_ingridients(
    _lock: &Mutex<Ingridients>,
    _cvar: &Condvar,
    _coffee_missing: &mut i32,
    _milk_missing: &mut i32,
) {
}
///Itera hasta conseguir todos los ingredientes necesarios para la orden.
/// Luego espera el tiempo de preparacion de la orden.
fn dispenser(order: &OrderFormat, ingridients_pair: &Arc<(Mutex<Ingridients>, Condvar)>) {
    let (lock, cvar) = &**ingridients_pair;
    print_mod(format!("preparando orden: {{{}}}", order));
    let mut coffee_missing = order.coffee;
    let mut milk_missing = order.foam;
    while coffee_missing > 0 || milk_missing > 0 {
        grab_ingridients(lock, cvar, &mut coffee_missing, &mut milk_missing);
    }
    print_mod("se termino de recojer ingredientes".to_string());

    //tiempo de preparacion
    ingridient_sleep(order.coffee);
    ingridient_sleep(order.foam);
    ingridient_sleep(order.hot_water);
}

///Ingresa las estadisticas de la orden procesada.
fn register_order(order: &OrderFormat, stats_lock: &Arc<RwLock<Stats>>) {
    let mut stats = stats_lock
        .write()
        .expect("no se pudo escribir en los stats");
    stats.c_consumed += order.coffee;
    stats.e_consumed += order.foam;
    stats.water_consumed += order.hot_water;
    stats.coffee_consumed += 1;
}

///Consume una orden del buffer cuando esta disponible.
/// Si el buffer esta en 0 y el productor termino de leer se alertan a las otras threads y termina esta funcion.
pub fn consumer(
    order_resources: Arc<ConsumerProducerOrders>,
    ingridients_pair: Arc<(Mutex<Ingridients>, Condvar)>,
    stats: Arc<RwLock<Stats>>,
) {
    let mut cond: State;
    let mut order: OrderFormat;
    {
        let stop_read = order_resources
            .stop
            .read()
            .expect("no se pudo leer en el stop");
        cond = *stop_read;
    }
    while !matches!(cond, State::FinishedProcessing) {
        order_resources.not_empty.acquire();
        {
            let mut buffer = order_resources
                .orders
                .write()
                .expect("no se pudo escribir en el buffer de ordenes");
            if buffer.len() == 0 {
                if matches!(cond, State::FinishedReading) {
                    print_mod("alertando".to_string());
                    let mut stop_write = order_resources
                        .stop
                        .write()
                        .expect("no se pudo escribir en el stop");
                    *stop_write = State::FinishedProcessing;
                }
                {
                    let stop_read = order_resources
                        .stop
                        .read()
                        .expect("no se pudo leer en el stop");
                    cond = *stop_read;
                }
                continue;
            }
            order = buffer.remove(0);
            order_resources.not_full.release();
        }
        dispenser(&order, &ingridients_pair);
        register_order(&order, &stats);
        {
            let stop_read = order_resources
                .stop
                .read()
                .expect("no se pudo leer en el stop");
            cond = *stop_read;
        }
    }
    print_mod("fin de consumidor".to_string());
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use crate::{
        consumer::{grab_ingridients, register_order},
        sync::{thread, Arc, Condvar, Mutex, RwLock},
        types::order_format::OrderFormat,
        Ingridients, Stats,
    };

    #[test]
    fn test_register_order() {
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

        let register_thread = thread::spawn(move || {
            let order = OrderFormat {
                coffee: 15,
                hot_water: 7,
                foam: 4,
            };
            register_order(&order, &stats_lock);
        });

        register_thread
            .join()
            .expect("no se pudo joinear el thread register");

        let stats_mut = stats_ref.read().expect("no se pudo leer stats");
        assert_eq!(15, stats_mut.c_consumed);
        assert_eq!(7, stats_mut.water_consumed);
        assert_eq!(4, stats_mut.e_consumed);
        assert_eq!(1, stats_mut.coffee_consumed);
        assert_eq!(0, stats_mut.g_consumed);
        assert_eq!(0, stats_mut.l_consumed);
    }

    fn test_grab_ingridients(
        num_coffee: i32,
        num_milk: i32,
        added_coffee: i32,
        added_milk: i32,
        should_miss_coffee: i32,
        should_miss_milk: i32,
        should_remain_coffee: i32,
        should_remain_milk: i32,
    ) {
        let ingridients = Ingridients {
            g: 0,
            c: 0,
            l: 0,
            e: 0,
        };
        let ingridients_pair = Arc::new((Mutex::new(ingridients), Condvar::new()));
        let ingridients_pair_clone = ingridients_pair.clone();
        let grab_thread = thread::spawn(move || {
            let (lock, cvar) = &*ingridients_pair_clone;
            let mut coffee_missing = num_coffee;
            let mut milk_missing = num_milk;
            grab_ingridients(lock, cvar, &mut coffee_missing, &mut milk_missing);
            assert_eq!(should_miss_coffee, coffee_missing);
            assert_eq!(should_miss_milk, milk_missing);
        });

        let (lock, cvar) = &*ingridients_pair;
        {
            let mut ingridients = lock.lock().expect("no se pudo agarrar los ingredientes");
            assert_eq!(0, ingridients.c);
            assert_eq!(0, ingridients.g);
            assert_eq!(0, ingridients.l);
            assert_eq!(0, ingridients.e);
            ingridients.c = added_coffee;
            ingridients.e = added_milk;
        }
        cvar.notify_all();
        grab_thread.join().expect("no se pudo joinear el thread");
        {
            let ingridients = lock.lock().expect("no se pudo agarrar los ingredientes");
            assert_eq!(should_remain_coffee, ingridients.c);
            assert_eq!(0, ingridients.g);
            assert_eq!(0, ingridients.l);
            assert_eq!(should_remain_milk, ingridients.e);
        }
    }

    #[test]
    fn test_grab_ingridients_all_grabed() {
        test_grab_ingridients(10, 10, 100, 100, 0, 0, 90, 90);
    }

    #[test]
    fn test_grab_ingridients_cant_grab_all() {
        test_grab_ingridients(20, 15, 10, 10, 10, 5, 0, 0);
    }

    #[test]
    fn test_grab_ingridients_exact() {
        test_grab_ingridients(10, 10, 10, 10, 0, 0, 0, 0);
    }

    #[test]
    fn test_grab_one_available() {
        test_grab_ingridients(10, 10, 10, 0, 0, 10, 0, 0);
    }

    #[test]
    fn test_grab_one_missing() {
        test_grab_ingridients(0, 10, 10, 10, 0, 0, 10, 0);
    }
}

#[cfg(test)]
#[cfg(loom)]
mod tests {
    use crate::{
        consumer::{grab_ingridients, register_order},
        sync::{thread, Arc, Condvar, Mutex, RwLock},
        types::order_format::OrderFormat,
        Ingridients, Stats,
    };

    #[test]
    fn test_register_order() {
        loom::model(|| {
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

            let register_thread = thread::spawn(move || {
                let order = OrderFormat {
                    coffee: 15,
                    hot_water: 7,
                    foam: 4,
                };
                register_order(&order, &stats_lock);
            });

            register_thread
                .join()
                .expect("no se pudo joinear el thread register");

            let stats_mut = stats_ref.read().expect("no se pudo leer stats");
            assert_eq!(15, stats_mut.c_consumed);
            assert_eq!(7, stats_mut.water_consumed);
            assert_eq!(4, stats_mut.e_consumed);
            assert_eq!(1, stats_mut.coffee_consumed);
            assert_eq!(0, stats_mut.g_consumed);
            assert_eq!(0, stats_mut.l_consumed);
        });
    }
}

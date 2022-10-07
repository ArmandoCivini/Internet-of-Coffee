use crate::sync::{thread, Arc, Condvar, Mutex, RwLock};
use std_semaphore::Semaphore;

mod types;
use types::consumer_producer_orders::ConsumerProducerOrders;
use types::ingridients::Ingridients;
use types::state::State;
use types::stats::Stats;

mod consumer;
use consumer::consumer;

mod producer;
use producer::producer;

mod ingridient_reloader;
use crate::ingridient_reloader::ingridient_reloader;

mod display_stats;
use crate::display_stats::display_stats;

mod print_mod;

mod sync {
    use std::time::Duration;

    #[cfg(not(loom))]
    pub(crate) use std::sync::{Arc, Condvar, Mutex, RwLock};

    #[cfg(loom)]
    pub(crate) use loom::sync::{Arc, Mutex, RwLock};

    #[cfg(loom)]
    pub(crate) use std::sync::Condvar;

    #[cfg(not(loom))]
    pub(crate) use std::thread;

    #[cfg(loom)]
    pub(crate) use loom::thread;

    #[cfg(not(loom))]
    pub(crate) fn sleep(d: Duration) {
        thread::sleep(d);
    }

    #[cfg(loom)]
    pub(crate) fn sleep(_d: Duration) {
        loom::thread::yield_now();
    }
}

fn main() {
    ioc_start("./orders/ordenes1.csv");
}

#[allow(clippy::needless_collect)]
///Comienza la ejecución del programa.
/// Recibe como parametro el archivo con las ordenes.
fn ioc_start(orders_file: &str) {
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

    let dispensers_threads: Vec<_> = (0..dispensers_number)
        .map(|_| {
            let consumer_producer_orders_clone = consumer_producer_orders_ref.clone();
            let ingridients_pair_clone = ingridients_pair.clone();
            let stats_clone = stats_ref.clone();
            thread::spawn(move || {
                consumer(
                    consumer_producer_orders_clone,
                    ingridients_pair_clone,
                    stats_clone,
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

    producer(consumer_producer_orders_ref, orders_file);

    let end_of_orders_clone_second = end_of_orders.clone();
    let stats_thread = thread::spawn(move || {
        display_stats(end_of_orders_clone_second, stats_ref);
    });

    dispensers_threads
        .into_iter()
        .flat_map(|x| x.join())
        .for_each(drop);
    {
        let mut stop_reloader = end_of_orders.write().expect("no se pudo escribir en stop");
        *stop_reloader = true;
    }
    let (lock, cvar) = &*ingridients_pair;
    {
        //desbloquea la condvar del reloader
        let mut ingridientss = lock
            .lock()
            .expect("no se pudo conseguir el mutex de ingredientes");
        ingridientss.c = 0;
    }
    cvar.notify_all();
    realoder_thread
        .join()
        .expect("no se pudo joinear la thread del recargador");
    stats_thread
        .join()
        .expect("no se pudo joinear la thread de estadisticas");
}

#[cfg(not(loom))]
#[cfg(test)]
//Test integral
mod tests {
    use serial_test::serial;
    use std::fs::read_to_string;
    use std::fs::File;
    use std::time::Duration;

    use crate::ioc_start;
    use crate::sync::sleep;

    #[test]
    #[serial]
    /// Se nesecita correr el test serialmente sino otro test puede pisar los contenidos de log
    fn test_full() {
        sleep(Duration::from_millis(100));
        // Se nesecita tiempo para flushear los test anteriores,
        // sino puede ser inpreciso el output.
        File::create("./log/log").expect("no se pudo crear el archivo");

        ioc_start("./orders/ordenes1.csv");

        //Obtiene el stdout
        let contents = read_to_string("./log/log").expect("Should have been able to read the file");

        general_test_output(&contents);

        //Ver si los stats son los correctos, solo chequeo el ultimo print de stats
        let char_pos = contents.rfind('{').expect("no se encontraron los stats");
        let (_, stats) = contents.split_at(char_pos);
        assert!(stats.contains("granos usados:10, cafe usado:150, leche usada:10, espuma usada:150, agua usada:100, cafe tomado:50}"));

        //Ver si se alerto del porcentaje de granos de cafe correctamente
        assert!(contents.contains("capacidad de ganos de cafe por debajo del 91%"));

        //Ver si se alerto del porcentaje de leche fria correctamente
        assert!(contents.contains("capacidad de leche fria por debajo del 91%"));

        //Ver si todas las ordenes se procesaron
        let all_orders_processed = contents
            .matches("preparando orden: {cafe:3, agua:2, espuma:3}")
            .count();
        assert_eq!(50, all_orders_processed);
    }

    #[test]
    #[serial]
    fn test_uneven() {
        sleep(Duration::from_millis(100));
        // Se nesecita tiempo para flushear los test anteriores,
        // sino puede ser inpreciso el output.
        File::create("./log/log").expect("no se pudo crear el archivo");

        ioc_start("./orders/ordenes3.csv");

        //Obtiene el stdout
        let contents = read_to_string("./log/log").expect("Should have been able to read the file");

        general_test_output(&contents);

        //Ver si los stats son los correctos, solo chequeo el ultimo print de stats
        let char_pos = contents.rfind('{').expect("no se encontraron los stats");
        let (_, stats) = contents.split_at(char_pos);
        assert!(stats.contains("granos usados:0, cafe usado:72, leche usada:0, espuma usada:77, agua usada:54, cafe tomado:10}"));

        //Ver si todas las ordenes se procesaron
        let all_orders_processed = contents.matches("preparando orden:").count();
        assert_eq!(10, all_orders_processed);
    }

    fn general_test_output(contents: &String) {
        //Ver si todas las threads consumidor cerraron
        let all_threads_closed = contents.matches("fin de consumidor").count();
        assert_eq!(10, all_threads_closed);

        //Ver si el productor cerro correctamente
        assert!(contents.contains("apagando productor"));

        //Ver si el recargador cerro correctamente
        assert!(contents.contains("Apagando recargador"));
    }
}

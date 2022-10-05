use crate::sync::Arc;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::print_mod::print_mod;
use crate::types::consumer_producer_orders::ConsumerProducerOrders;
use crate::types::order_format::OrderFormat;
use crate::types::state::State;

///Esta funcion se obtuvo de <https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html>
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// este modulo se encarga de leer ordenes del archivo y agregarlo al buffer protegido por semaforos.
/// Se comporta como el productor de un patron productor-consumidor
pub fn producer(order_resources: Arc<ConsumerProducerOrders>, path: &str) {
    if let Ok(lines) = read_lines(path) {
        for order in lines.flatten() {
            order_resources.not_full.acquire();
            let amounts: Vec<&str> = order.split(',').collect();
            let resource = OrderFormat {
                coffee: amounts[0]
                    .parse()
                    .expect("una de las ordenes no era numerica"),
                hot_water: amounts[1]
                    .parse()
                    .expect("una de las ordenes no era numerica"),
                foam: amounts[2]
                    .parse()
                    .expect("una de las ordenes no era numerica"),
            };
            {
                let mut buffer = order_resources
                    .orders
                    .write()
                    .expect("el productor no pudo escribir en el buffer");
                buffer.push(resource);
            }
            order_resources.not_empty.release();
        }
    }
    let mut stop_write = order_resources
        .stop
        .write()
        .expect("el productor no pudo escribir en el estado");
    *stop_write = State::FinishedReading;
    print_mod("apagando productor".to_string());
    for _i in 0..100 {
        //desbloquea threads en espera
        order_resources.not_empty.release();
    }
}

#[cfg(test)]
mod tests {
    use crate::sync::{thread, Arc, RwLock};
    use std::fs::File;
    use std_semaphore::Semaphore;

    use crate::producer;
    use crate::types::consumer_producer_orders::ConsumerProducerOrders;
    use crate::types::order_format::OrderFormat;
    use crate::types::state::State;

    #[test]
    #[cfg(not(loom))]
    fn test_all_is_read_small_buffer() {
        test_all_is_read(20, "./orders/ordenes1.csv", 3, 2, 3, 50);
    }

    #[test]
    #[cfg(not(loom))]
    fn test_all_is_read_big_buffer() {
        test_all_is_read(100, "./orders/ordenes1.csv", 3, 2, 3, 50);
    }

    #[test]
    #[cfg(loom)]
    fn test_all_is_read_loom() {
        loom::model(move || {
            test_all_is_read(20, "./orders/ordenes3.csv", 20, 10, 20, 1);
        });
    }

    fn test_all_is_read(
        orders_buffer_size: isize,
        orders_file: &'static str,
        coffee: i32,
        water: i32,
        milk: i32,
        lines: i32,
    ) {
        //crea el archivo de logs si no existe, si existe lo trunca
        File::create("./log/log").expect("no se pudo crear el archivo");
        let mut order_num = 0;
        let stop = RwLock::new(State::Reading);
        let consumer_producer_orders = ConsumerProducerOrders {
            not_empty: Semaphore::new(0),
            not_full: Semaphore::new(orders_buffer_size),
            orders: RwLock::new(Vec::new()),
            stop: stop,
        };
        let consumer_producer_orders_ref = Arc::new(consumer_producer_orders);

        let consumer_producer_orders_clone = consumer_producer_orders_ref.clone();
        let producer_thread = thread::spawn(move || {
            producer::producer(consumer_producer_orders_clone, orders_file);
        });

        let mut cond: State;
        let mut order: OrderFormat;
        {
            let stop_read = consumer_producer_orders_ref
                .stop
                .read()
                .expect("no se pudo leer en el stop");
            cond = *stop_read;
        }
        while !matches!(cond, State::FinishedProcessing) {
            consumer_producer_orders_ref.not_empty.acquire();
            {
                let mut buffer = consumer_producer_orders_ref
                    .orders
                    .write()
                    .expect("no se pudo escribir en el buffer de ordenes");
                if buffer.len() == 0 {
                    if matches!(cond, State::FinishedReading) {
                        let mut stop_write = consumer_producer_orders_ref
                            .stop
                            .write()
                            .expect("no se pudo escribir en el stop");
                        *stop_write = State::FinishedProcessing;
                    }
                    {
                        let stop_read = consumer_producer_orders_ref
                            .stop
                            .read()
                            .expect("no se pudo leer en el stop");
                        cond = *stop_read;
                    }
                    continue;
                }
                order = buffer.remove(0);
                assert_eq!(coffee, order.coffee);
                assert_eq!(water, order.hot_water);
                assert_eq!(milk, order.foam);
                order_num += 1;
                consumer_producer_orders_ref.not_full.release();
            }
            {
                let stop_read = consumer_producer_orders_ref
                    .stop
                    .read()
                    .expect("no se pudo leer en el stop");
                cond = *stop_read;
            }
        }

        producer_thread
            .join()
            .expect("error al joinear producer thread");
        assert_eq!(lines, order_num);
    }

    #[test]
    #[cfg(loom)]
    fn loom_test() {

        loom::model(move || {
            let mut order: OrderFormat;
            let stop = RwLock::new(State::Reading);
            let consumer_producer_orders = ConsumerProducerOrders {
                not_empty: Semaphore::new(0),
                not_full: Semaphore::new(20),
                orders: RwLock::new(Vec::new()),
                stop: stop,
            };
            let consumer_producer_orders_ref = Arc::new(consumer_producer_orders);
    
            let consumer_producer_orders_clone = consumer_producer_orders_ref.clone();
            let producer_thread = thread::spawn(move || {
                producer::producer(consumer_producer_orders_clone, "./orders/ordenes3.csv");
            });
            consumer_producer_orders_ref.not_empty.acquire();
            {
                let mut buffer = consumer_producer_orders_ref
                    .orders
                    .write()
                    .expect("no se pudo escribir en el buffer de ordenes");
                order = buffer.remove(0);
                assert_eq!(20, order.coffee);
                assert_eq!(10, order.hot_water);
                assert_eq!(20, order.foam);
                consumer_producer_orders_ref.not_full.release();
            }
            producer_thread.join();
        });
    }
}

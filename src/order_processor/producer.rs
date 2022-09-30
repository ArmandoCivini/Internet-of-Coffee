use crate::print_mod::print_mod::print_mod;
use crate::types::consumer_producer_orders::ConsumerProducerOrders;
use crate::types::order_format::OrderFormat;
use crate::types::state::State;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::Arc;

///Esta funcion se obtuvo de https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
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
        for line in lines {
            if let Ok(order) = line {
                order_resources.not_full.acquire();
                let amounts: Vec<&str> = order.split(",").collect();
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
    }
    let mut stop_write = order_resources
        .stop
        .write()
        .expect("el productor no pudo escribir en el estado");
    *stop_write = State::FinishedReading;
    print_mod(format!("apagando productor"));
    for _i in 0..100 {
        //desbloquea threads en espera
        order_resources.not_empty.release();
    }
}

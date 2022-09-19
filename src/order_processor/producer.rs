use crate::types::state::State;
use std::sync::Arc;
use crate::types::consumer_producer_orders::ConsumerProducerOrders;

pub fn producer(
    order_resources: Arc<ConsumerProducerOrders>
) {
    let mut i = 0;
    while i < 50 {
        order_resources.not_full.acquire();
        {
            let mut buffer = order_resources.orders.write().unwrap();
            buffer.push(i);
        }
        order_resources.not_empty.release();
        i += 1;
    }
    //orders_not_full.acquire();
    let mut stop_write = order_resources.stop.write().unwrap();
    *stop_write = State::FinishedReading;
    println!("finished producer");
    for _i in 0..100 {
        order_resources.not_empty.release();
    }
}

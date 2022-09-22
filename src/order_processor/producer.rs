use crate::types::consumer_producer_orders::ConsumerProducerOrders;
use crate::types::order_format::OrderFormat;
use crate::types::state::State;
use std::sync::Arc;

pub fn producer(order_resources: Arc<ConsumerProducerOrders>) {
    let mut i = 0;
    while i < 50 {
        order_resources.not_full.acquire();
        {
            let resource = OrderFormat {
                coffee: 3,
                hot_water: 2,
                foam: 4,
            };
            let mut buffer = order_resources.orders.write().unwrap();
            buffer.push(resource);
        }
        order_resources.not_empty.release();
        i += 1;
    }
    let mut stop_write = order_resources.stop.write().unwrap();
    *stop_write = State::FinishedReading;
    println!("finished producer");
    for _i in 0..100 {
        // unlocks waiting threads
        order_resources.not_empty.release();
    }
}

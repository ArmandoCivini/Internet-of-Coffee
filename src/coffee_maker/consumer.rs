use crate::types::state::State;
use std::sync::Arc;
use crate::types::consumer_producer_orders::ConsumerProducerOrders;

pub fn consumer(
    order_resources: Arc<ConsumerProducerOrders>
) {
    let mut cond: State;
    {
        let stop_read = order_resources.stop.read().unwrap();
        cond = *stop_read;
    }
    while !matches!(cond, State::FinishedProcessing) {
        order_resources.not_empty.acquire();
        {
            let mut buffer = order_resources.orders.write().unwrap();
            if buffer.len() == 0 {
                if matches!(cond, State::FinishedReading) {
                    println!("alerting");
                    let mut stop_write = order_resources.stop.write().unwrap();
                    *stop_write = State::FinishedProcessing;
                }
                {
                    let stop_read = order_resources.stop.read().unwrap();
                    cond = *stop_read;
                }
                continue;
            }
            println!("{}", buffer.remove(0));
            order_resources.not_full.release();
        }
        {
            let stop_read = order_resources.stop.read().unwrap();
            cond = *stop_read;
        }
    }
    println!("finished consumer");
}

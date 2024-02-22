use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

mod event;
mod kafka;

const ORDER_PLACED_EVENT: &str = "order_placed";
const BROKER: &str = "localhost:9092";

#[derive(Debug, Serialize, Deserialize)]
struct NewOrderPlaced {
    price_in_ticks: u64,
    base_lots_placed: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fetcher = crate::event::EventFetcher::new().await?;
    let mut event_stream = fetcher.run();

    while let Some(event) = event_stream.next().await {
        println!("Received a Place event: {:?}\n", event);
        println!("Sending to Kafka...\n");

        let message = serde_json::to_string(&event)?;

        crate::kafka::produce_message(BROKER, ORDER_PLACED_EVENT, &message).await;
    }

    Ok(())
}

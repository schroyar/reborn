use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

mod event;

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
    }

    Ok(())
}

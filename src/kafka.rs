use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub async fn produce_message(brokers: &str, topic_name: &str, message: &str) {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .expect("Producer creation error");

    let record = FutureRecord::to(topic_name)
        .payload(message)
        .key("rust_key");

    match producer.send(record, Duration::from_secs(0)).await {
        Ok(delivery) => println!("Message delivered: {:?}", delivery),
        Err((e, _)) => println!("Message delivery failed: {:?}", e),
    }
}

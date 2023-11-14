use actors::{batch::BatchActor, messages::BatchMessage, state::StateActor};
use rdkafka::{
    consumer::{CommitMode, Consumer, StreamConsumer},
    ClientConfig, Message,
};
use tokio::sync::mpsc;

use crate::{actors::messages::StateMessage, model::Transaction};

mod actors;
mod model;

#[tokio::main]
async fn main() {
    let (state_tx, state_rx) = mpsc::channel::<StateMessage>(1);
    let (batch_tx, batch_rx) = mpsc::channel::<BatchMessage>(1);

    tokio::spawn(async move {
        let state_actor = StateActor::new(state_rx, batch_tx);
        state_actor.run().await;
    });

    tokio::spawn(async move {
        let batch_actor = BatchActor::new(batch_rx);
        batch_actor.run().await;
    });

    let transactions_str = "transactions";

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "transaction_group")
        .set("bootstrap.servers", "localhost:29092")
        .set("enable.auto.commit", "true")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[transactions_str])
        .expect("Topic subscription failed");

    println!("Waiting for messages");

    loop {
        match consumer.recv().await {
            Ok(message) => {
                let payload = match message.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };

                let transaction: Transaction =
                    serde_json::from_str(payload).expect("Error deserializing the message");
                println!("Received transaction: {:?}", transaction);

                let state_message = StateMessage {
                    single_data: transaction,
                };

                state_tx.send(state_message).await.unwrap();

                consumer
                    .commit_message(&message, CommitMode::Async)
                    .unwrap();
            }
            Err(e) => println!("Kafka error: {}", e),
        }
    }
}

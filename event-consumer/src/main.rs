use couchbase::{Cluster, QueryOptions};
use futures::stream::StreamExt;
use rdkafka::{
    consumer::{CommitMode, Consumer, StreamConsumer},
    ClientConfig, Message,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    pub id: u64,
    pub user_id: u64,
    pub amount: f64,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Serialize, Deserialize)]
enum TransactionType {
    Bet,
    Trade,
    Deposit,
    Withdrawal,
}

#[tokio::main]
async fn main() {
    let cluster = Cluster::connect("couchbase://localhost:8091", "Administrator", "password");
    let bucket = cluster.bucket("transactions");
    let collection = bucket.default_collection();

    let topic = "transactions";

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "transaction_group")
        .set("bootstrap.servers", "localhost:29092")
        .set("enable.auto.commit", "true")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[topic])
        .expect("Topic subscription failed");

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

                collection
                    .upsert(transaction.id.to_string(), &transaction, None)
                    .await
                    .expect("Error upserting transaction");

                println!("Transaction inserted");

                match collection.get(transaction.id.to_string(), None).await {
                    Ok(db_transaction) => {
                        println!("Transaction from db: {:?}", db_transaction);
                    }
                    Err(e) => {
                        println!("Get error: {:?}", e)
                    }
                }

                consumer
                    .commit_message(&message, CommitMode::Async)
                    .unwrap();
            }
            Err(e) => println!("Kafka error: {}", e),
        }
    }
}

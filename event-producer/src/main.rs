use tokio::time::Duration;

use rand::Rng;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use serde::{Deserialize, Serialize};

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
    let topic = "transactions";

    loop {
        let transaction = generate_transaction();

        if let Err(e) = produce_event(&transaction, topic).await {
            eprintln!("Error producing transaction: {}", e);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn generate_transaction() -> Transaction {
    let mut rng = rand::thread_rng();

    let transaction_type = match rng.gen_range(0..4) {
        0 => TransactionType::Bet,
        1 => TransactionType::Trade,
        2 => TransactionType::Deposit,
        _ => TransactionType::Withdrawal,
    };

    Transaction {
        id: rng.gen_range(1..1000000000),
        user_id: rng.gen_range(1..1000000000),
        amount: rng.gen_range(1.0..1000.0),
        transaction_type,
    }
}

async fn produce_event(
    transaction: &Transaction,
    topic: &str,
) -> Result<(), rdkafka::error::KafkaError> {
    //TODO: We create client for each produce event,
    // Should be created once
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:29092")
        .create()
        .expect("Producer creation error");

    let payload = serde_json::to_string(&transaction).unwrap();

    let status = producer
        .send(
            FutureRecord::to(topic)
                .payload(&payload)
                .key(&transaction.id.to_string()),
            Duration::from_secs(0),
        )
        .await;

    println!("Send event status: {:?}, payload: {}", status, payload);

    Ok(())
}

use couchbase::{Cluster, QueryOptions};
use tokio::sync::mpsc::Receiver;

use crate::actors::messages::BatchMessage;

pub struct BatchActor {
    pub cluster: Cluster,
    pub receiver: Receiver<BatchMessage>,
}

impl BatchActor {
    pub fn new(receiver: Receiver<BatchMessage>) -> BatchActor {
        let cluster = Cluster::connect("couchbase://127.0.0.1:8091", "Administrator", "password");

        BatchActor { cluster, receiver }
    }

    async fn handle_message(&mut self, message: BatchMessage) {
        println!("Batch actor received a message");

        let bucket_name = "transactions";
        let scope_name = "transactions";
        let collection_name = message.data_type;

        let mut values = Vec::new();
        for transaction in message.batch_data {
            let key = format!("\"{}\"", transaction.id);
            let value = serde_json::to_string(&transaction).unwrap();
            values.push(format!("({}, {})", key, value));
        }

        let values_str = values.join(", ");

        let query = format!(
            "INSERT INTO `{}`.`{}`.`{}` (KEY, VALUE) VALUES {}",
            bucket_name, scope_name, collection_name, values_str
        );

        self.cluster
            .query(query, QueryOptions::default())
            .await
            .expect("INSERT query failed");
    }

    pub async fn run(mut self) {
        println!("Batch actor is running");
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await
        }
    }
}

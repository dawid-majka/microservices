use std::{collections::HashMap, time::Duration};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::interval,
};

use crate::actors::messages::{BatchMessage, StateMessage};
use crate::model::{Transaction, TransactionType};

const INTERVAL: u64 = 60;
const MAX_CACHE: usize = 100;

pub struct StateActor {
    pub cache: HashMap<TransactionType, Vec<Transaction>>,
    pub receiver: Receiver<StateMessage>,
    pub sender: Sender<BatchMessage>,
}

impl StateActor {
    pub fn new(receiver: Receiver<StateMessage>, sender: Sender<BatchMessage>) -> StateActor {
        let cache: HashMap<TransactionType, Vec<Transaction>> = HashMap::new();

        StateActor {
            cache,
            receiver,
            sender,
        }
    }

    async fn handle_message(&mut self, message: StateMessage) {
        println!("State actor received a message");

        let key = message.single_data.transaction_type.clone();

        self.cache.entry(key.clone()).or_insert_with(Vec::new);

        if let Some(transactions) = self.cache.get_mut(&key) {
            transactions.push(message.single_data);

            if transactions.len() >= MAX_CACHE {
                self.flush_cache_bucket(key).await;
            }
        }
    }

    async fn flush_cache_bucket(&mut self, key: TransactionType) {
        if let Some(transactions) = self.cache.get_mut(&key) {
            let batch_data = std::mem::take(transactions);

            let message = BatchMessage {
                data_type: key,
                batch_data,
            };
            let _ = self.sender.send(message).await;
        }
    }

    async fn flush_cache(&mut self) {
        for (k, v) in self.cache.drain().collect::<HashMap<_, _>>() {
            let message = BatchMessage {
                data_type: k,
                batch_data: v,
            };

            let _ = self.sender.send(message).await;
        }
    }

    pub async fn run(mut self) {
        println!("State actor is running");

        let mut interval_timer = interval(Duration::from_secs(INTERVAL));
        loop {
            tokio::select! {
                _ = interval_timer.tick() => {
                    self.flush_cache().await;
                },
                Some(msg) = self.receiver.recv() => {
                    self.handle_message(msg).await
                }
            }
        }
    }
}

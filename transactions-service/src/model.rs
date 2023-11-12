use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouchbaseTransactionWrapper {
    #[serde(flatten)]
    pub inner: HashMap<String, Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub user_id: u64,
    pub amount: f64,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Bet,
    Trade,
    Deposit,
    Withdrawal,
}

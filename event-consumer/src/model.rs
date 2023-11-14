use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub user_id: u64,
    pub amount: f64,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum TransactionType {
    Bet,
    Trade,
    Deposit,
    Withdrawal,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransactionType::Bet => write!(f, "bet"),
            TransactionType::Trade => write!(f, "trade"),
            TransactionType::Deposit => write!(f, "deposit"),
            TransactionType::Withdrawal => write!(f, "withdrawal"),
        }
    }
}

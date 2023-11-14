use crate::model::{Transaction, TransactionType};

pub struct StateMessage {
    pub single_data: Transaction,
}

pub struct BatchMessage {
    pub data_type: TransactionType,
    pub batch_data: Vec<Transaction>,
}

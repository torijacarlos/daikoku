use chrono::{Utc, DateTime};

use super::TransactionType;

#[derive(Debug)]
pub struct Transaction {
    pub id: Option<u32>,
    pub amount: f32,
    pub execution_date: DateTime<Utc>,
    pub trx_type: TransactionType,
    pub account_id: u32,
}

impl Transaction {
    pub fn new(account_id: u32, amount: f32, trx_type: TransactionType) -> Self {
        Transaction {
            id: None,
            amount,
            account_id,
            trx_type,
            execution_date: Utc::now(),
        }
    }
}



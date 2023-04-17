use chrono::{Utc, DateTime};

use super::{AccountType, TransactionType, Transaction};

#[derive(Debug)]
pub struct Account {
    pub id: Option<u32>,
    pub wallet_id: u32,
    pub name: String,
    pub acc_type: AccountType,
    pub created_date: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
}

impl Account {
    pub fn create(wallet_id: u32, name: String, acc_type: AccountType) -> Self {
        // (torijacarlos:todo): Store immediately to database, and return instance
        Self {
            id: None,
            name,
            acc_type,
            wallet_id,
            created_date: Utc::now(),
            transactions: vec![],
        }
    }

    pub fn balance(&self) -> f32 {
        let mut total: f32 = 0.0;
        let multiplier = match &self.acc_type {
            AccountType::Asset | AccountType::Expense => 1.0,
            _ => -1.0,
        };
        for trx in &self.transactions {
            match trx.trx_type {
                TransactionType::Debit => total += trx.amount * multiplier,
                TransactionType::Credit => total -= trx.amount * multiplier,
            }
        }
        total
    }
}



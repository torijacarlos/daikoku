use chrono::{Utc, DateTime};

use super::{AccountType, TransactionType, Transaction};

#[derive(Debug)]
pub struct Account<'a> {
    pub id: Option<u32>,
    pub wallet_id: u32,
    pub name: &'a str,
    pub acc_type: AccountType,
    pub created_date: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
}

impl<'a> Account<'a> {
    pub fn new(wallet_id: u32, name: &'a str, acc_type: AccountType) -> Self {
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



use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{AccountType, Transaction, TransactionType};
use num_traits::cast::ToPrimitive;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub acc_type: AccountType,

    pub balance: BigDecimal,
    pub balance_date: DateTime<Utc>,
    pub transactions: Vec<Transaction>,

    pub created_date: Option<DateTime<Utc>>,
    pub updated_date: Option<DateTime<Utc>>,
}

pub fn get_account_balance(acc: &Account) -> f32 {
    let mut total: f32 = acc.balance.to_f32().unwrap();
    let multiplier = match acc.acc_type {
        AccountType::Asset | AccountType::Expense => 1.0,
        _ => -1.0,
    };
    let transactions = acc
        .transactions
        .iter()
        .filter(|t| t.execution_date >= acc.balance_date);
    for trx in transactions {
        match trx.trx_type {
            TransactionType::Debit => total += trx.amount.to_f32().unwrap() * multiplier,
            TransactionType::Credit => total -= trx.amount.to_f32().unwrap() * multiplier,
        }
    }
    total
}

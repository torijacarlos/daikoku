use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{get_account_balance, Account, AccountType};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Wallet {
    pub alias: String,
    pub accounts: Vec<Account>,

    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
}

pub fn get_accounts_net_worth(accounts: &Vec<Account>) -> f32 {
    let mut total = 0.0;
    for acc in accounts {
        total += get_account_balance(acc);
    }
    total
}

pub fn get_wallet_liquidity_index(accounts: &Vec<Account>) -> f32 {
    let mut num = 0.0;
    let mut dem = 0.0;
    for acc in accounts {
        match acc.acc_type {
            AccountType::Asset | AccountType::Expense => {
                num += get_account_balance(acc);
            }
            _ => {
                dem += get_account_balance(acc);
            }
        }
    }
    if dem == 0.0 {
        return f32::INFINITY;
    }
    (num / dem).abs()
}

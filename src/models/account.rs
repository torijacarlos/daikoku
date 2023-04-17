use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::{types::BigDecimal, MySql, Pool};

use crate::{alias::DkkResult, error::DkkError};

use super::{AccountType, Transaction, TransactionType};
use num_traits::cast::ToPrimitive;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Account {
    // db data
    pub id: Option<u32>,
    pub created_date: Option<DateTime<Utc>>,
    pub updated_date: Option<DateTime<Utc>>,

    // data
    pub wallet_id: u32,
    pub name: String,
    pub acc_type: AccountType,

    pub balance: BigDecimal,
    pub balance_date: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
}

impl Account {
    pub async fn upsert(&self, pool: &Pool<MySql>) -> DkkResult<()> {
        let acc_type = sqlx::query!(
            "SELECT id FROM LU_ACCOUNT_TYPE WHERE value = ?",
            self.acc_type.as_str()
        )
        .fetch_one(&mut pool.acquire().await?)
        .await?;
        if self.id.is_some() {
            sqlx::query!(
                r#"
                UPDATE ACCOUNT 
                SET name = ?, type_id = ?, balance = ?, balance_date = ?
                WHERE id = ?"#,
                self.name,
                acc_type.id,
                self.balance,
                self.balance_date,
                self.id
            )
            .execute(&mut pool.acquire().await?)
            .await?;
        } else {
            sqlx::query!(
                r#"INSERT INTO ACCOUNT 
                (wallet_id, name, type_id, balance, balance_date) 
                VALUES (?, ?, ?, ?, ?)"#,
                self.wallet_id,
                self.name,
                acc_type.id,
                self.balance,
                self.balance_date,
            )
            .execute(&mut pool.acquire().await?)
            .await?;
        }
        Ok(())
    }
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

pub async fn get_account_transactions(
    account_id: u32,
    pool: &Pool<MySql>,
) -> DkkResult<Vec<Transaction>> {
    sqlx::query_as!(
        Transaction,
        r#"SELECT  
        t.id as "id?", amount, execution_date, lu.value as "trx_type: TransactionType", account_id
        FROM TRANSACTION t
        JOIN LU_TRANSACTION_TYPE lu 
        ON t.type_id = lu.id
        WHERE t.account_id = ?"#,
        account_id
    )
    .fetch_all(&mut pool.acquire().await?)
    .await
    .map_err(DkkError::Database)
}

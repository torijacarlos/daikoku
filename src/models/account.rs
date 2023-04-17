use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool};

use crate::{alias::DaikokuResult, error::DaikokuError};

use super::{AccountType, Transaction, TransactionType};
use num_traits::cast::ToPrimitive;

#[derive(Debug)]
pub struct Account {
    pub id: u32,
    pub wallet_id: u32,
    pub name: String,
    pub acc_type: AccountType,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
}

impl Account {
    pub async fn create(
        wallet_id: u32,
        name: String,
        acc_type: AccountType,
        pool: &Pool<MySql>,
    ) -> DaikokuResult<Self> {
        let result = sqlx::query!(
            r#"SELECT id FROM LU_ACCOUNT_TYPE WHERE value = ?"#,
            acc_type.as_str()
        )
        .fetch_one(&mut pool.acquire().await?)
        .await?;
        let result = sqlx::query!(
            r#"INSERT INTO ACCOUNT (wallet_id, name, type_id) VALUES (?, ?, ?)"#,
            wallet_id,
            name,
            result.id
        )
        .execute(&mut pool.acquire().await?)
        .await?;

        Self::get(result.last_insert_id() as u32, pool).await
    }

    pub async fn get(id: u32, pool: &Pool<MySql>) -> DaikokuResult<Self> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT 
            a.id, name, wallet_id, created_date, updated_date, lu.value as "acc_type: AccountType"
            FROM ACCOUNT a 
            JOIN LU_ACCOUNT_TYPE lu 
            ON a.type_id = lu.id
            WHERE a.id = ?"#,
            id
        )
        .fetch_one(&mut pool.acquire().await?)
        .await
        .map_err(DaikokuError::DatabaseError)
    }

    pub async fn save(&self, pool: &Pool<MySql>) -> DaikokuResult<()> {
        let acc_type = sqlx::query!(
            "select id from LU_ACCOUNT_TYPE where value=?",
            self.acc_type.as_str()
        )
        .fetch_one(&mut pool.acquire().await?)
        .await?;

        sqlx::query!(
            r#"
            UPDATE ACCOUNT 
            SET name = ?, type_id = ?
            WHERE id = ?"#,
            self.name,
            acc_type.id,
            self.id
        )
        .execute(&mut pool.acquire().await?)
        .await?;

        Ok(())
    }
}

pub fn get_account_balance(
    acc_type: &AccountType,
    transactions: &Vec<Transaction>,
) -> DaikokuResult<f32> {
    let mut total: f32 = 0.0;
    let multiplier = match acc_type {
        AccountType::Asset | AccountType::Expense => 1.0,
        _ => -1.0,
    };
    for trx in transactions.iter() {
        match trx.trx_type {
            TransactionType::Debit => total += trx.amount.to_f32().unwrap() * multiplier,
            TransactionType::Credit => total -= trx.amount.to_f32().unwrap() * multiplier,
        }
    }
    Ok(total)
}

pub async fn get_account_transactions(
    account_id: u32,
    pool: &Pool<MySql>,
) -> DaikokuResult<Vec<Transaction>> {
    sqlx::query_as!(
        Transaction,
        r#"SELECT  
        t.id, amount, execution_date, lu.value as "trx_type: TransactionType", account_id
        FROM TRANSACTION t
        JOIN LU_TRANSACTION_TYPE lu 
        ON t.type_id = lu.id
        WHERE t.account_id = ?"#,
        account_id
    )
    .fetch_all(&mut pool.acquire().await?)
    .await
    .map_err(DaikokuError::DatabaseError)
}

use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool};

use crate::{alias::DkkResult, error::DkkError};

use super::{AccountType, Transaction, TransactionType};
use num_traits::cast::ToPrimitive;

#[derive(Debug, Default, Clone)]
pub struct Account {
    // db data
    pub id: Option<u32>,
    pub created_date: Option<DateTime<Utc>>,
    pub updated_date: Option<DateTime<Utc>>,

    // data
    pub wallet_id: u32,
    pub name: String,
    pub acc_type: AccountType,
    pub transactions: Vec<Transaction>,
}

impl Account {
    pub async fn create(
        wallet_id: u32,
        name: String,
        acc_type: AccountType,
        pool: &Pool<MySql>,
    ) -> DkkResult<Self> {
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

    pub async fn get(id: u32, pool: &Pool<MySql>) -> DkkResult<Self> {
        let account_row = sqlx::query!(
            r#"
            SELECT 
            a.id as "id?", name, wallet_id, created_date as "created_date?", updated_date as "updated_date?", lu.value as "acc_type: AccountType"
            FROM ACCOUNT a 
            JOIN LU_ACCOUNT_TYPE lu 
            ON a.type_id = lu.id
            WHERE a.id = ?"#,
            id
        )
        .fetch_one(&mut pool.acquire().await?)
        .await
        .map_err(DkkError::Database)?;

        Ok(Account {
            id: account_row.id,
            created_date: account_row.created_date,
            updated_date: account_row.updated_date,
            wallet_id: account_row.wallet_id,
            name: account_row.name.clone(),
            acc_type: account_row.acc_type,
            transactions: vec![],
        })
    }

    pub async fn save(&self, pool: &Pool<MySql>) -> DkkResult<()> {
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

pub fn get_account_balance(acc: &Account) -> f32 {
    let mut total: f32 = 0.0;
    let multiplier = match acc.acc_type {
        AccountType::Asset | AccountType::Expense => 1.0,
        _ => -1.0,
    };
    for trx in acc.transactions.iter() {
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

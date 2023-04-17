use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool};

use super::{AccountType, Transaction, TransactionType};

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
        pool: &mut Pool<MySql>,
    ) -> Result<Self, sqlx::Error> {
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

    pub async fn get(id: u32, pool: &mut Pool<MySql>) -> Result<Self, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT 
            a.id, name, wallet_id, created_date, updated_date, lu.value as acc_type
            FROM ACCOUNT a 
            JOIN LU_ACCOUNT_TYPE lu 
            ON a.type_id = lu.id
            WHERE a.id = ?"#,
            id
        )
        .fetch_one(&mut pool.acquire().await?)
        .await?;

        Ok(Self {
            id: result.id,
            wallet_id: result.wallet_id,
            name: result.name,
            acc_type: match TryInto::<AccountType>::try_into(result.acc_type) {
                Ok(v) => v,
                Err(_) => return Err(sqlx::Error::RowNotFound),
            },
            created_date: result.created_date,
            updated_date: result.updated_date,
        })
    }

    pub fn save(&self, _pool: &mut Pool<MySql>) -> Result<(), sqlx::Error> {
        todo!()
    }

    pub fn get_transactions(&self, _pool: &mut Pool<MySql>) -> Vec<Transaction> {
        todo!()
    }

    pub fn balance(&self, pool: &mut Pool<MySql>) -> f32 {
        let mut total: f32 = 0.0;
        let multiplier = match &self.acc_type {
            AccountType::Asset | AccountType::Expense => 1.0,
            _ => -1.0,
        };
        for trx in &self.get_transactions(pool) {
            match trx.trx_type {
                TransactionType::Debit => total += trx.amount * multiplier,
                TransactionType::Credit => total -= trx.amount * multiplier,
            }
        }
        total
    }
}

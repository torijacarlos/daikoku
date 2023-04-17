use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool};

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
    }

    pub async fn save(&self, pool: &mut Pool<MySql>) -> Result<(), sqlx::Error> {
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

    pub async fn get_transactions(
        &self,
        pool: &mut Pool<MySql>,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        sqlx::query_as!(
            Transaction,
            r#"SELECT  
            t.id, amount, execution_date, lu.value as "trx_type: TransactionType", account_id
            FROM TRANSACTION t
            JOIN LU_TRANSACTION_TYPE lu 
            ON t.type_id = lu.id
            WHERE account_id = ?"#,
            self.id
        )
        .fetch_all(&mut pool.acquire().await?)
        .await
    }

    pub async fn balance(&self, pool: &mut Pool<MySql>) -> Result<f32, sqlx::Error> {
        let mut total: f32 = 0.0;
        let multiplier = match &self.acc_type {
            AccountType::Asset | AccountType::Expense => 1.0,
            _ => -1.0,
        };
        for trx in self.get_transactions(pool).await?.iter() {
            match trx.trx_type {
                TransactionType::Debit => total += trx.amount.to_f32().unwrap() * multiplier,
                TransactionType::Credit => total -= trx.amount.to_f32().unwrap() * multiplier,
            }
        }
        Ok(total)
    }
}

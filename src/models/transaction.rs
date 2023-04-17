use chrono::{DateTime, Utc};
use sqlx::{types::BigDecimal, MySql, Pool};

use crate::{alias::DkkResult, error::DkkError};

use super::TransactionType;

#[derive(Debug)]
pub struct Transaction {
    pub id: u32,
    pub amount: BigDecimal,
    pub execution_date: DateTime<Utc>,
    pub trx_type: TransactionType,
    pub account_id: u32,
}

impl Transaction {
    pub async fn create(
        account_id: u32,
        amount: f32,
        trx_type: TransactionType,
        pool: &Pool<MySql>,
    ) -> DkkResult<Self> {
        let result = sqlx::query!(
            r#"SELECT id FROM LU_TRANSACTION_TYPE WHERE value = ?"#,
            trx_type.as_str()
        )
        .fetch_one(&mut pool.acquire().await?)
        .await?;
        let result = sqlx::query!(
            r#"INSERT INTO TRANSACTION (account_id, amount, type_id) VALUES (?, ?, ?)"#,
            account_id,
            amount,
            result.id
        )
        .execute(&mut pool.acquire().await?)
        .await?;

        Self::get(result.last_insert_id() as u32, pool).await
    }

    pub async fn get(id: u32, pool: &Pool<MySql>) -> DkkResult<Self> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT 
            t.id, amount, execution_date, lu.value as "trx_type: TransactionType", account_id
            FROM TRANSACTION t
            JOIN LU_TRANSACTION_TYPE lu
            ON lu.id = t.type_id
            WHERE t.id = ?"#,
            id
        )
        .fetch_one(&mut pool.acquire().await?)
        .await
        .map_err(DkkError::Database)
    }

    pub async fn save(&self, pool: &Pool<MySql>) -> DkkResult<()> {
        let trx_type = sqlx::query!(
            "select id from LU_TRANSACTION_TYPE where value=?",
            self.trx_type.as_str()
        )
        .fetch_one(&mut pool.acquire().await?)
        .await?;

        sqlx::query!(
            r#"
            UPDATE TRANSACTION 
            SET amount = ?, execution_date = ?, type_id = ?
            WHERE id = ?"#,
            self.amount,
            self.execution_date,
            trx_type.id,
            self.id
        )
        .execute(&mut pool.acquire().await?)
        .await?;

        Ok(())
    }
}

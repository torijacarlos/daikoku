use chrono::{DateTime, Utc};
use sqlx::{types::BigDecimal, MySql, Pool};

use crate::{alias::DkkResult, error::DkkError};

use super::TransactionType;

#[derive(Debug, Default, Clone)]
pub struct Transaction {
    // db data
    pub id: Option<u32>,

    // data
    pub account_id: u32,
    pub amount: BigDecimal,
    pub execution_date: DateTime<Utc>,
    pub trx_type: TransactionType,
}

impl Transaction {
    pub async fn upsert(&self, pool: &Pool<MySql>) -> DkkResult<Self> {
        let id: u32;
        let trx_type = sqlx::query!(
            "SELECT id FROM LU_TRANSACTION_TYPE WHERE value = ?",
            self.trx_type.as_str()
        )
        .fetch_one(&mut pool.acquire().await?)
        .await?;
        if self.id.is_some() {
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
            id = self.id.unwrap();
        } else {
            let result = sqlx::query!(
                r#"INSERT INTO TRANSACTION (account_id, amount, type_id) VALUES (?, ?, ?)"#,
                self.account_id,
                self.amount,
                trx_type.id
            )
            .execute(&mut pool.acquire().await?)
            .await?;
            id = result.last_insert_id() as u32;
        }
        Self::get(id, pool).await
    }

    pub async fn get(id: u32, pool: &Pool<MySql>) -> DkkResult<Self> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT 
            t.id as "id?", amount, execution_date, lu.value as "trx_type: TransactionType", account_id
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
}

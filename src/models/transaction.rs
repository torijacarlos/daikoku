use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::BigDecimal, MySql, Pool};

use crate::alias::DkkResult;

use super::TransactionType;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
    pub async fn upsert(&mut self, pool: &Pool<MySql>) -> DkkResult<()> {
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
        } else {
            let result = sqlx::query!(
                r#"INSERT INTO TRANSACTION (account_id, amount, type_id) VALUES (?, ?, ?)"#,
                self.account_id,
                self.amount,
                trx_type.id
            )
            .execute(&mut pool.acquire().await?)
            .await?;
            self.id = result.last_insert_id().try_into().ok();
        }
        Ok(())
    }
}

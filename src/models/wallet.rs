use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool};

use super::{Account, AccountType};

#[derive(Debug)]
pub struct Wallet {
    pub id: u32,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
}

impl Wallet {
    pub async fn create(pool: &mut Pool<MySql>) -> Result<Self, sqlx::Error> {
        let result = sqlx::query!(r#"INSERT INTO WALLET () VALUES ()"#)
            .execute(&mut pool.acquire().await?)
            .await?;

        Self::get(result.last_insert_id() as u32, pool).await
    }

    pub async fn get(id: u32, pool: &mut Pool<MySql>) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, created_date, updated_date FROM WALLET WHERE id = ?"#,
            id
        )
        .fetch_one(&mut pool.acquire().await?)
        .await
    }

    pub async fn get_accounts(&self, pool: &mut Pool<MySql>) -> Result<Vec<Account>, sqlx::Error> {
        sqlx::query_as!(
            Account,
            r#"SELECT  
            a.id, name, wallet_id, created_date, updated_date, lu.value as "acc_type: AccountType"
            FROM ACCOUNT a
            JOIN LU_ACCOUNT_TYPE lu 
            ON a.type_id = lu.id
            WHERE wallet_id = ?"#,
            self.id
        )
        .fetch_all(&mut pool.acquire().await?)
        .await
    }

    pub async fn net_worth(&self, pool: &mut Pool<MySql>) -> Result<f32, sqlx::Error> {
        let mut total = 0.0;
        for acc in self.get_accounts(pool).await? {
            total += &acc.balance(pool).await?;
        }
        Ok(total)
    }
}

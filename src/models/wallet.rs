use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool};

use crate::{alias::DkkResult, error::DkkError};

use super::{get_account_balance, Account, AccountType, Transaction};

#[derive(Debug)]
pub struct Wallet {
    pub id: u32,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub accounts: HashMap<Account, Vec<Transaction>>,
}

unsafe impl Send for Wallet {}

impl Wallet {
    pub async fn create(pool: &Pool<MySql>) -> DkkResult<Self> {
        let result = sqlx::query!(r#"INSERT INTO WALLET () VALUES ()"#)
            .execute(&mut pool.acquire().await?)
            .await?;

        Self::get(result.last_insert_id() as u32, pool).await
    }

    pub async fn get(id: u32, pool: &Pool<MySql>) -> DkkResult<Self> {
        let wallet = sqlx::query!(
            r#"SELECT id, created_date, updated_date FROM WALLET WHERE id = ?"#,
            id
        )
        .fetch_one(&mut pool.acquire().await?)
        .await?;

        Ok(Self {
            id: wallet.id,
            created_date: wallet.created_date,
            updated_date: wallet.updated_date,
            accounts: HashMap::new(),
        })
    }
}

pub async fn get_wallet_accounts(wallet_id: u32, pool: &Pool<MySql>) -> DkkResult<Vec<Account>> {
    sqlx::query_as!(
        Account,
        r#"SELECT  
            a.id, name, wallet_id, created_date, updated_date, lu.value as "acc_type: AccountType"
            FROM ACCOUNT a
            JOIN LU_ACCOUNT_TYPE lu 
            ON a.type_id = lu.id
            WHERE wallet_id = ?"#,
        wallet_id
    )
    .fetch_all(&mut pool.acquire().await?)
    .await
    .map_err(DkkError::Database)
}

pub fn get_accounts_net_worth(accounts: &HashMap<Account, Vec<Transaction>>) -> f32 {
    let mut total = 0.0;
    for acc in accounts.keys() {
        total += if let Some(t) = accounts.get(acc) {
            get_account_balance(&acc.acc_type, t)
        } else {
            0.0
        };
    }
    total
}

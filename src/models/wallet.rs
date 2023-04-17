use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool};

use crate::{alias::DkkResult, error::DkkError};

use super::{get_account_balance, Account, AccountType};

#[derive(Debug)]
pub struct Wallet {
    // db data
    pub id: u32,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub accounts: Vec<Account>,
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
            accounts: vec![],
        })
    }
}

pub async fn get_all_wallet_ids(pool: &Pool<MySql>) -> DkkResult<Vec<u32>> {
    sqlx::query!(r#"SELECT id FROM WALLET"#)
        .fetch_all(&mut pool.acquire().await?)
        .await
        .map(|res| res.iter().map(|v| v.id).collect())
        .map_err(DkkError::Database)
}

pub async fn get_wallet_accounts(wallet_id: u32, pool: &Pool<MySql>) -> DkkResult<Vec<Account>> {
    sqlx::query!(
        r#"SELECT  
            a.id as "id?", name, wallet_id, created_date as "created_date?", updated_date as "updated_date?", lu.value as "acc_type: AccountType"
            FROM ACCOUNT a
            JOIN LU_ACCOUNT_TYPE lu 
            ON a.type_id = lu.id
            WHERE wallet_id = ?"#,
        wallet_id
    )
    .fetch_all(&mut pool.acquire().await?)
    .await
    .map(|result| {
        result.iter().map(|account| {
            Account {
                id: account.id,
                created_date: account.created_date,
                updated_date: account.updated_date,
                wallet_id: account.wallet_id,
                name: account.name.clone(),
                acc_type: account.acc_type.clone(),
                transactions: vec![]
            }
        })
        .collect()
    })
    .map_err(DkkError::Database)
}

pub fn get_accounts_net_worth(accounts: &Vec<Account>) -> f32 {
    let mut total = 0.0;
    for acc in accounts {
        total += get_account_balance(acc);
    }
    total
}

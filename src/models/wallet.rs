use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};

use crate::{alias::DkkResult, error::DkkError};

use super::{get_account_balance, Account, AccountType};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Wallet {
    // db data
    pub id: Option<u32>,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub accounts: Vec<Account>,
}

unsafe impl Send for Wallet {}

impl Wallet {
    pub async fn upsert(&mut self, pool: &Pool<MySql>) -> DkkResult<()> {
        if self.id.is_some() {
            sqlx::query!(
                r#"
                UPDATE WALLET 
                SET id = ?
                WHERE id = ?"#,
                self.id,
                self.id
            )
            .execute(&mut pool.acquire().await?)
            .await?;
        } else {
            let result = sqlx::query!(r#"INSERT INTO WALLET () VALUES ()"#)
                .execute(&mut pool.acquire().await?)
                .await?;
            self.id = result.last_insert_id().try_into().ok();
        }
        Ok(())
    }

    pub async fn get(id: u32, pool: &Pool<MySql>) -> DkkResult<Self> {
        let wallet = sqlx::query!(
            r#"SELECT id, created_date, updated_date FROM WALLET WHERE id = ?"#,
            id
        )
        .fetch_one(&mut pool.acquire().await?)
        .await?;

        Ok(Self {
            id: Some(wallet.id),
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
            a.id as "id?", name, wallet_id, 
            lu.value as "acc_type: AccountType",
            balance, balance_date,
            created_date as "created_date?", updated_date as "updated_date?"
            FROM ACCOUNT a
            JOIN LU_ACCOUNT_TYPE lu 
            ON a.type_id = lu.id
            WHERE wallet_id = ?"#,
        wallet_id
    )
    .fetch_all(&mut pool.acquire().await?)
    .await
    .map(|result| {
        result
            .iter()
            .map(|account| Account {
                id: account.id,
                created_date: account.created_date,
                updated_date: account.updated_date,
                wallet_id: account.wallet_id,
                name: account.name.clone(),
                acc_type: account.acc_type.clone(),
                balance: account.balance.clone(),
                balance_date: account.balance_date,
                transactions: vec![],
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

pub fn get_wallet_liquidity_index(accounts: &Vec<Account>) -> f32 {
    let mut num = 0.0;
    let mut dem = 0.0;
    for acc in accounts {
        match acc.acc_type {
            AccountType::Asset | AccountType::Expense => {
                num += get_account_balance(acc);
            }
            _ => {
                dem += get_account_balance(acc);
            }
        }
    }
    if dem == 0.0 {
        return f32::INFINITY;
    }
    (num / dem).abs()
}

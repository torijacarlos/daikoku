use sqlx::{MySql, Pool};

use crate::{
    alias::DkkResult,
    models::{get_account_transactions, get_all_wallet_ids, get_wallet_accounts, Wallet},
    ui::DkkUiState,
    Dkk,
};

pub fn load(app: &mut Dkk) {
    if let DkkUiState::Init = app.state {
        load_available_wallets(app);
    }
    if let Some(wallet_id) = app.working_wallet {
        load_wallet(app, wallet_id);
    }
}

pub fn export(wallet: &Wallet) {
    let mut location = home::home_dir().unwrap();
    location.push("atelier");
    let _ = std::fs::create_dir(&location);
    location.push(".daikoku");
    let wallet_string = ron::to_string(wallet);
    if let Ok(ws) = wallet_string {
        let _ = std::fs::write(location, ws);
    }
}

pub async fn import(pool: &Pool<MySql>) -> DkkResult<()> {
    let mut location = home::home_dir().unwrap();
    location.push("atelier");
    location.push(".daikoku");
    // @todo: this is nasty. Do only three queries. One insert for the wallet, one batch insert
    // for accounts, and one for transactions
    if let Ok(wallet_string) = std::fs::read_to_string(location) {
        let mut wallet: Wallet = ron::from_str(&wallet_string).unwrap();
        wallet.id = None;
        wallet.upsert(pool).await?;
        for mut acc in wallet.accounts {
            acc.id = None;
            acc.upsert(pool).await?;
            for mut t in acc.transactions {
                t.id = None;
                t.upsert(pool).await?;
            }
        }
    }
    Ok(())
}

fn load_available_wallets(app: &mut Dkk) {
    let av_wallets_ref = app.available_wallets.clone();
    let pool_ref = app.pool.clone();

    tokio::spawn(async move {
        let wallets = get_all_wallet_ids(&pool_ref).await.ok();

        if let Ok(mut guard) = av_wallets_ref.lock() {
            *guard = wallets;
        }
    });
}

fn load_wallet(app: &mut Dkk, wallet_id: u32) {
    let wallet_ref = app.wallet.clone();
    let pool_ref = app.pool.clone();

    if !app.wallet.is_set() || app.force_reload {
        app.force_reload = false;
        tokio::spawn(async move {
            let mut wallet = Wallet::get(wallet_id, &pool_ref).await.ok();
            if let Some(ref mut wallet) = wallet {
                if let Ok(mut accounts) = get_wallet_accounts(wallet_id, &pool_ref).await {
                    for acc in &mut accounts {
                        let ts = get_account_transactions(acc.id.unwrap(), &pool_ref).await;
                        acc.transactions = if let Ok(ts) = ts { ts } else { vec![] };
                    }
                    wallet.accounts = accounts;
                }
            }

            if let Ok(mut wallet_guard) = wallet_ref.lock() {
                *wallet_guard = wallet;
            }
        });
    }
}

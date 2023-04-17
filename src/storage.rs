use crate::{
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

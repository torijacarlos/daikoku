use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
    Nonce, // Or `Aes128Gcm`
};
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

//let key = Aes256Gcm::generate_key(&mut OsRng);
//let cipher = Aes256Gcm::new(&key);
//let nonce = Nonce::from_slice(&pin[..]); // 96-bits; unique per message
//let ciphertext = cipher.encrypt(nonce, wallet_string.as_ref())?;
//let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;

fn left_pad(pin: &String, len: usize) -> String {
    if pin.len() < len {
        let mut pad = (0..(len - pin.len())).map(|_| " ").collect::<String>();
        pad.push_str(pin);
        return pad;
    }
    pin.clone()
}

pub fn export(wallet: &Wallet, pin: &String) {
    let wallet = clear_ids(wallet.clone());
    let mut location = home::home_dir().unwrap();
    location.push("atelier");
    let _ = std::fs::create_dir(&location);
    location.push(".daikoku");
    let wallet_string = ron::to_string(&wallet);
    if let Ok(ws) = wallet_string {
        let cipher = Aes256Gcm::new(left_pad(&"daikoku".to_string(), 32).as_bytes().into());
        let pin = left_pad(pin, 12);
        let nonce = Nonce::from_slice(pin.as_bytes()); // 96-bits; unique per message
        if let Ok(ciphertext) = cipher.encrypt(nonce, ws.as_bytes()) {
            let _ = std::fs::write(location, ciphertext);
        }
    }
}

fn clear_ids(mut wallet: Wallet) -> Wallet {
    wallet.id = None;
    for mut acc in &mut wallet.accounts {
        acc.id = None;
        acc.wallet_id = 0;
        for mut t in &mut acc.transactions {
            t.id = None;
            t.account_id = 0;
        }
    }
    wallet
}

pub async fn import(pool: &Pool<MySql>, pin: &String) -> DkkResult<()> {
    let cipher = Aes256Gcm::new(left_pad(&"daikoku".to_string(), 32).as_bytes().into());
    let pin = left_pad(pin, 12);
    let nonce = Nonce::from_slice(pin.as_bytes()); // 96-bits; unique per message
    let mut location = home::home_dir().unwrap();
    location.push("atelier");
    location.push(".daikoku");
    // @todo: this is nasty. Do only three queries. One insert for the wallet, one batch insert
    // for accounts, and one for transactions
    match std::fs::read(location) {
        Ok(wallet_string) => match cipher.decrypt(nonce, wallet_string.as_slice()) {
            Ok(wallet_string) => {
                if let Ok(wallet_string) = std::str::from_utf8(&wallet_string) {
                    let wallet: Wallet = ron::from_str(wallet_string).unwrap();
                    println!("{:#?}", wallet);
                    let mut wallet = clear_ids(wallet.clone());
                    wallet.upsert(pool).await?;
                    for mut acc in wallet.accounts {
                        acc.wallet_id = wallet.id.unwrap();
                        acc.upsert(pool).await?;
                        for mut t in acc.transactions {
                            t.account_id = acc.id.unwrap();
                            t.upsert(pool).await?;
                        }
                    }
                }
            }
            Err(e) => println!("{}", e),
        },
        Err(e) => println!("{}", e),
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

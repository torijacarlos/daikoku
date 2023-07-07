pub mod alias;
pub mod error;
mod models;
mod settings;
mod storage;

use std::path::PathBuf;

use models::Wallet;
use settings::Settings;
use uuid::Uuid;

#[derive(Debug)]
pub struct Dkk {
    pub pin: String,
    pub wallet: Option<Wallet>,

    pub available_wallets: Vec<PathBuf>,

    pub working_alias: String,
    pub crypt_key: String,

    pub working_account_id: Option<Uuid>,
    pub working_transaction_id: Option<Uuid>,
}

impl Dkk {
    pub fn new() -> Self {
        let settings = Settings::load().unwrap();
        Self {
            pin: String::new(),
            wallet: None,
            available_wallets: storage::get_all_wallets_locations(),
            working_alias: String::new(),
            working_account_id: None,
            working_transaction_id: None,
            crypt_key: settings.crypt_key,
        }
    }
}


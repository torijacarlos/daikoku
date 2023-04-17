mod alias;
mod error;
mod models;
mod settings;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use alias::{DaikokuResult, DaikokuThreadData};
use eframe::egui;
use egui::RichText;
use error::DaikokuError;
use models::{get_account_transactions, get_wallet_accounts, Account, Transaction};
use sqlx::{MySql, Pool};

use crate::models::Wallet;
use crate::settings::Settings;

struct Daikoku {
    wallet: DaikokuThreadData<Wallet>,

    // @todo: maybe both accounts and transactions should be a single hashmap
    // get_accounts_net_worth could receive HashMap<Account, Vec<Transaction>> instead
    accounts: DaikokuThreadData<Vec<Account>>,
    transactions: DaikokuThreadData<HashMap<u32, Vec<Transaction>>>,
    pool: Arc<Pool<MySql>>,
    frame: u16,
    fps: u16,
    start_time: Instant,
}

impl Daikoku {
    fn new() -> Self {
        let settings = Settings::load().unwrap();
        let pool = Arc::new(settings.get_db_conn_pool());
        Self {
            wallet: DaikokuThreadData::empty(),
            accounts: DaikokuThreadData::empty(),
            transactions: DaikokuThreadData::new(HashMap::new()),
            pool,
            frame: 0,
            fps: 0,
            start_time: Instant::now(),
        }
    }
}

fn load_wallet(app: &Daikoku, wallet_id: u32) {
    let wallet_ref = app.wallet.clone();
    let pool_ref = app.pool.clone();
    if !app.wallet.is_set() {
        tokio::spawn(async move {
            let wallet = Wallet::get(wallet_id, &pool_ref).await.ok();
            if let Ok(mut wallet_guard) = wallet_ref.lock() {
                *wallet_guard = wallet;
            }
        });
    }
}

fn load_accounts(app: &Daikoku, wallet_id: u32) {
    let pool_ref = app.pool.clone();
    let accounts_ref = app.accounts.clone();

    if app.wallet.is_set() && !app.accounts.is_set() {
        tokio::spawn(async move {
            let accounts = get_wallet_accounts(wallet_id, &pool_ref).await.ok();
            if let Ok(mut accounts_guard) = accounts_ref.lock() {
                *accounts_guard = accounts;
            }
        });
    }
}

fn load_transactions(app: &Daikoku, account_id: u32) {
    let pool_ref = app.pool.clone();
    let transactions_ref = app.transactions.clone();

    tokio::spawn(async move {
        let mut loaded_transactions = false;
        if let Ok(transactions_guard) = transactions_ref.lock() {
            if let Some(ref transactions) = *transactions_guard {
                loaded_transactions = transactions.contains_key(&account_id);
            }
        }

        if !loaded_transactions {
            let transactions = get_account_transactions(account_id, &pool_ref).await.ok();
            if let Some(transactions) = transactions {
                if let Ok(mut transactions_guard) = transactions_ref.lock() {
                    if let Some(ref mut transactions_map) = *transactions_guard {
                        transactions_map.insert(account_id, transactions);
                    }
                }
            }
        }
    });
}

fn clear_data(app: &Daikoku) {
    let wallet_ref = app.wallet.clone();
    let accounts_ref = app.accounts.clone();
    let transactions_ref = app.transactions.clone();
    tokio::spawn(async move {
        if let Ok(mut guard) = wallet_ref.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = accounts_ref.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = transactions_ref.lock() {
            *guard = Some(HashMap::new());
        }
    });
}

#[tokio::main]
async fn main() -> DaikokuResult<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Daikoku", options, Box::new(|_| Box::new(Daikoku::new())))
        .map_err(DaikokuError::RenderError)
}

impl eframe::App for Daikoku {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.frame += 1;

            ui.heading("Daikoku");

            // load data
            let wallet_id = 1;
            load_wallet(self, wallet_id);
            load_accounts(self, wallet_id);
            self.accounts.get(|accounts| {
                if let Some(accounts) = accounts {
                    for acc in accounts {
                        load_transactions(self, acc.id);
                    }
                }
            });

            // render data

            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.label(RichText::new("Wallet information").strong());
                    ui.horizontal(|ui| {
                        self.wallet.get(|w: Option<&Wallet>| {
                            if let Some(w) = w {
                                ui.label(format!("Id: {}", w.id));
                                ui.label(format!("Created date: {:?}", w.created_date));
                            }
                        });
                    });
                });
            });

            ui.vertical(|ui| {
                ui.label(RichText::new("Accounts").strong());

                self.accounts.get(|accounts| {
                    if let Some(accounts) = accounts {
                        for acc in accounts {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(format!("Account Id: {}", acc.id));
                                    ui.label(format!("Account type: {:?}", acc.acc_type));
                                    ui.label(format!(
                                        "Account Created date: {:?}",
                                        acc.created_date
                                    ));
                                    self.transactions.get(|transactions| {
                                        if let Some(transactions) = transactions {
                                            if transactions.contains_key(&acc.id) {
                                                if let Some(trxs) = transactions.get(&acc.id) {
                                                    for t in trxs {
                                                        ui.group(|ui| {
                                                            ui.label(format!(
                                                                "Transaction id: {}",
                                                                t.id
                                                            ));
                                                            ui.label(format!(
                                                                "Amount: {:?}",
                                                                t.amount
                                                            ));
                                                            ui.label(format!(
                                                                "Trx Type: {:?}",
                                                                t.trx_type
                                                            ));
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    });
                                });
                            });
                        }
                    }
                });
            });

            ui.group(|ui| {
                // see fps
                let sec_marker = self.start_time.elapsed().as_secs_f32() % 1.0;
                if sec_marker > 0.985 {
                    self.fps = self.frame;
                    self.frame = 0;
                    clear_data(self);
                }
                ctx.request_repaint();
                ui.label(format!("fps '{:?}'", self.fps));
            });
        });
    }
}

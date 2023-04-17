mod alias;
mod error;
mod models;
mod settings;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

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

    accounts: DaikokuThreadData<HashMap<Account, Vec<Transaction>>>,
    pool: Arc<Pool<MySql>>,
    frame: u128,
    frame_time: Instant,
    fps: f32,
}

impl Daikoku {
    fn new() -> Self {
        let settings = Settings::load().unwrap();
        let pool = Arc::new(settings.get_db_conn_pool());
        Self {
            wallet: DaikokuThreadData::empty(),
            accounts: DaikokuThreadData::new(HashMap::new()),
            pool,
            frame: 0,
            frame_time: Instant::now(),
            fps: 0.0,
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

    if app.wallet.is_set() {
        tokio::spawn(async move {
            let mut map: HashMap<Account, Vec<Transaction>> = HashMap::new();
            let accounts = get_wallet_accounts(wallet_id, &pool_ref).await.ok();
            if let Some(accounts) = accounts {
                for acc in accounts {
                    let trxs = get_account_transactions(acc.id, &pool_ref).await;
                    if let Ok(trxs) = trxs {
                        map.insert(acc, trxs);
                    }
                }

                if let Ok(mut accounts_guard) = accounts_ref.lock() {
                    *accounts_guard = Some(map);
                }
            }
        });
    }
}

fn clear_data(app: &Daikoku) {
    let wallet_ref = app.wallet.clone();
    let accounts_ref = app.accounts.clone();
    tokio::spawn(async move {
        if let Ok(mut guard) = wallet_ref.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = accounts_ref.lock() {
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
            ui.heading("Daikoku");

            // load data
            let wallet_id = 1;
            load_wallet(self, wallet_id);
            load_accounts(self, wallet_id);

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
                        for acc in accounts.keys() {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(format!("Account Id: {}", acc.id));
                                    ui.label(format!("Account type: {:?}", acc.acc_type));
                                    ui.label(format!(
                                        "Account Created date: {:?}",
                                        acc.created_date
                                    ));
                                    if let Some(transactions) = accounts.get(acc) {
                                        for t in transactions {
                                            ui.group(|ui| {
                                                ui.label(format!("Transaction id: {}", t.id));
                                                ui.label(format!("Amount: {:?}", t.amount));
                                                ui.label(format!("Trx Type: {:?}", t.trx_type));
                                            });
                                        }
                                    }
                                });
                            });
                        }
                    }
                });
            });

            ui.group(|ui| {
                // see fps
                ui.label(format!("fps '{:?}'", self.fps));
            });
        });

        self.frame += 1;
        if self.frame_time.elapsed() > Duration::new(1, 0) {
            self.fps = self.frame as f32;
            self.frame = 0;
            self.frame_time = Instant::now();
        }
        ctx.request_repaint();
    }
}

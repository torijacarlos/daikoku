mod alias;
mod error;
mod models;
mod settings;

use std::sync::Arc;
use std::time::{Duration, Instant};

use alias::{DaikokuResult, DaikokuThreadData};
use eframe::egui;
use egui::RichText;
use error::DaikokuError;
use models::{get_account_transactions, get_accounts_net_worth, get_wallet_accounts, Account};
use sqlx::{MySql, Pool};

use crate::models::Wallet;
use crate::settings::Settings;

struct Daikoku {
    wallet: DaikokuThreadData<Wallet>,

    pool: Arc<Pool<MySql>>,

    force_reload: bool,
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
            pool,
            force_reload: false,
            frame: 0,
            frame_time: Instant::now(),
            fps: 0.0,
        }
    }
}

fn load_wallet(app: &mut Daikoku, wallet_id: u32) {
    let wallet_ref = app.wallet.clone();
    let pool_ref = app.pool.clone();
    if !app.wallet.is_set() || app.force_reload {
        app.force_reload = false;
        tokio::spawn(async move {
            let mut wallet = Wallet::get(wallet_id, &pool_ref).await.ok();
            if let Some(ref mut wallet) = wallet {
                if let Ok(accounts) = get_wallet_accounts(wallet_id, &pool_ref).await {
                    for acc in accounts {
                        let ts = get_account_transactions(acc.id, &pool_ref).await;

                        wallet
                            .accounts
                            .insert(acc, if let Ok(ts) = ts { ts } else { vec![] });
                    }
                }
            }

            if let Ok(mut wallet_guard) = wallet_ref.lock() {
                *wallet_guard = wallet;
                println!("Reloaded!");
            }
        });
    }
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

            // render data
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.label(RichText::new("Wallet information").strong());
                    ui.vertical(|ui| {
                        self.wallet.get(|w: Option<&Wallet>| {
                            if let Some(w) = w {
                                ui.group(|ui| {
                                    ui.label(format!("Id: {}", w.id));
                                    ui.label(format!("Created date: {:?}", w.created_date));
                                    ui.label(format!(
                                        "Net Worth: {:?}",
                                        get_accounts_net_worth(&w.accounts)
                                    ));
                                });
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("Accounts").strong());
                                    let mut accounts: Vec<&Account> = w.accounts.keys().collect();
                                    accounts.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

                                    for acc in accounts {
                                        ui.group(|ui| {
                                            ui.vertical(|ui| {
                                                ui.label(format!("Account Id: {}", acc.id));
                                                ui.label(format!(
                                                    "Account type: {:?}",
                                                    acc.acc_type
                                                ));
                                                ui.label(format!(
                                                    "Account Created date: {:?}",
                                                    acc.created_date
                                                ));
                                                if let Some(transactions) = w.accounts.get(acc) {
                                                    for t in transactions {
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
                                            });
                                        });
                                    }
                                });
                            }
                        });
                    });
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
            self.force_reload = true;
        }
        ctx.request_repaint();
    }
}

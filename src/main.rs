mod alias;
mod error;
mod models;
mod settings;

use std::sync::Arc;
use std::time::{Duration, Instant};

use alias::{DkkResult, DkkThreadData};
use eframe::egui;
use egui::RichText;
use error::DkkError;
use models::{get_account_transactions, get_accounts_net_worth, get_wallet_accounts, Account};
use sqlx::{MySql, Pool};

use crate::models::Wallet;
use crate::settings::Settings;

enum DkkState {
    Wallet,
    CreateTransaction { account_id: u32, amount_str: String },
}

struct Dkk {
    wallet: DkkThreadData<Wallet>,
    pool: Arc<Pool<MySql>>,
    force_reload: bool,
    fps: f32,
    frame: u128,
    frame_time: Instant,
    state: DkkState,
}

impl Dkk {
    fn new() -> Self {
        let settings = Settings::load().unwrap();
        Self {
            wallet: DkkThreadData::empty(),
            pool: Arc::new(settings.get_db_conn_pool()),
            force_reload: false,
            fps: 0.0,
            frame: 0,
            state: DkkState::Wallet,
            frame_time: Instant::now(),
        }
    }
}

fn load_wallet(app: &mut Dkk, wallet_id: u32) {
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
            }
        });
    }
}

#[tokio::main]
async fn main() -> DkkResult<()> {
    eframe::run_native(
        "Dkk",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(Dkk::new())),
    )
    .map_err(DkkError::RenderError)
}

impl eframe::App for Dkk {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.heading("Dkk");
                ui.label(format!("fps: {:?}", self.fps));
            });

            // render state/scene
            match &mut self.state {
                DkkState::Wallet => {
                    // render data
                    ui.group(|ui| {
                        render_wallet(ui, self);
                    });
                }
                DkkState::CreateTransaction {
                    account_id,
                    amount_str,
                } => {
                    ui.group(|ui| {
                        ui.label(format!("Creating Transaction for account: {account_id}"));
                    });

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let label = ui.label(format!("Amount: "));
                            ui.text_edit_singleline(amount_str).labelled_by(label.id);
                        });
                    });
                }
            };

            // inputs
            if let DkkState::CreateTransaction { .. } = self.state {
                ui.input(|input| {
                    if input.key_pressed(egui::Key::Escape) {
                        self.state = DkkState::Wallet;
                    }
                });
            };
        });

        let wallet_id = 1;
        let seconds = 2;
        self.frame += 1;
        if self.frame_time.elapsed() > Duration::new(seconds, 0) {
            self.fps = self.frame as f32 / (seconds as f32);
            self.frame = 0;
            self.frame_time = Instant::now();
            self.force_reload = true;
        }

        load_wallet(self, wallet_id);
        ctx.request_repaint();
    }
}

fn render_wallet(ui: &mut egui::Ui, app: &mut Dkk) {
    app.wallet.get(|wallet: Option<&Wallet>| {
        if let Some(wallet) = wallet {
            ui.vertical(|ui| {
                ui.label(RichText::new("Wallet information").strong());
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label(format!("Id: {}", wallet.id));
                        ui.label(format!("Created date: {:?}", wallet.created_date));
                        ui.label(format!(
                            "Net Worth: {:?}",
                            get_accounts_net_worth(&wallet.accounts)
                        ));
                    });
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Accounts").strong());
                        let mut accounts: Vec<&Account> = wallet.accounts.keys().collect();
                        accounts.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

                        for acc in accounts {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(format!("Account Id: {}", acc.id));
                                    ui.label(format!("Account type: {:?}", acc.acc_type));
                                    ui.label(format!(
                                        "Account Created date: {:?}",
                                        acc.created_date
                                    ));
                                    if let Some(transactions) = wallet.accounts.get(acc) {
                                        for t in transactions {
                                            ui.group(|ui| {
                                                ui.label(format!("Transaction id: {}", t.id));
                                                ui.label(format!("Amount: {:?}", t.amount));
                                                ui.label(format!("Trx Type: {:?}", t.trx_type));
                                            });
                                        }
                                        ui.group(|ui| {
                                            if ui.button("Create transaction").clicked() {
                                                ui.label("Creating transaction");
                                                app.state = DkkState::CreateTransaction {
                                                    account_id: acc.id,
                                                    amount_str: String::new(),
                                                }
                                            }
                                        });
                                    }
                                });
                            });
                        }
                    });
                });
            });
        }
    });
}

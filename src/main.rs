mod alias;
mod error;
mod models;
mod settings;

use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use alias::{DkkResult, DkkThreadData};
use eframe::egui;
use egui::RichText;
use error::DkkError;
use models::{
    get_account_balance, get_account_transactions, get_accounts_net_worth, get_all_wallet_ids,
    get_wallet_accounts, Account, AccountType, Transaction, TransactionType,
};
use sqlx::types::BigDecimal;
use sqlx::{MySql, Pool};

use crate::models::Wallet;
use crate::settings::Settings;

enum DkkState {
    Init,
    WalletView,
    AccountView,
    TransactionView,
}

struct Dkk {
    wallet: DkkThreadData<Wallet>,

    available_wallets: DkkThreadData<Vec<u32>>,

    working_wallet: Option<u32>,
    working_account: Account,
    working_transaction: Transaction,

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
            state: DkkState::Init,
            available_wallets: DkkThreadData::empty(),
            working_wallet: None,
            wallet: DkkThreadData::empty(),
            working_account: Account::default(),
            working_transaction: Transaction::default(),
            pool: Arc::new(settings.get_db_conn_pool()),
            force_reload: false,
            fps: 0.0,
            frame: 0,
            frame_time: Instant::now(),
        }
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

#[tokio::main]
async fn main() -> DkkResult<()> {
    eframe::run_native(
        "Dkk",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(Dkk::new())),
    )
    .map_err(DkkError::Render)
}

impl eframe::App for Dkk {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.heading("Dkk");
                ui.label(format!("fps: {:?}", self.fps));
            });

            render(ui, self);
            handle_input(ui, self);
        });

        if let DkkState::Init = self.state {
            load_available_wallets(self);
        }
        if let Some(wallet_id) = self.working_wallet {
            load_wallet(self, wallet_id);
        }
        update_fps(self);
        ctx.request_repaint();
    }
}

fn update_fps(app: &mut Dkk) {
    let seconds = 2;
    app.frame += 1;
    if app.frame_time.elapsed() > Duration::new(seconds, 0) {
        app.fps = app.frame as f32 / (seconds as f32);
        app.frame = 0;
        app.frame_time = Instant::now();
        app.force_reload = true;
    }
}

fn render(ui: &mut egui::Ui, app: &mut Dkk) {
    match app.state {
        DkkState::Init => render_init(ui, app),
        DkkState::WalletView => render_wallet(ui, app),
        DkkState::AccountView => render_account(ui, app),
        DkkState::TransactionView => render_transaction(ui, app),
    };
}

fn render_init(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label("Select or create a wallet");
        ui.vertical(|ui| {
            app.available_wallets.get(|aw| {
                if let Some(aw) = aw {
                    for wallet_id in aw {
                        if ui.button(format!("{wallet_id}")).clicked() {
                            app.working_wallet = Some(*wallet_id);
                            app.state = DkkState::WalletView;
                        }
                    }
                }
            });
        });
        ui.horizontal(|ui| {
            if ui.button("Create").clicked() {
                let pool_ref = app.pool.clone();
                tokio::spawn(async move {
                    if Wallet::upsert(&pool_ref).await.is_err() {
                        todo!("unhandled error");
                    }
                });
            }
        });
    });
}

fn render_account(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label(format!(
            "{} Account for Wallet: {}",
            if app.working_account.id.is_some() {
                "Editing"
            } else {
                "Creating"
            },
            app.working_account.wallet_id
        ));
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            let label = ui.label("Name: ".to_string());
            ui.text_edit_singleline(&mut app.working_account.name)
                .labelled_by(label.id);
        });
        ui.horizontal(|ui| {
            let label = ui.label("Type: ".to_string());
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", app.working_account.acc_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Asset,
                        "Asset",
                    );
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Equity,
                        "Equity",
                    );
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Expense,
                        "Expense",
                    );
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Income,
                        "Income",
                    );
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Liability,
                        "Liability",
                    );
                })
                .response
                .labelled_by(label.id);
        });
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                app.state = DkkState::WalletView;
                let pool_ref = app.pool.clone();
                let ca_copy = app.working_account.clone();
                tokio::spawn(async move {
                    if ca_copy.upsert(&pool_ref).await.is_err() {
                        todo!("unhandled error");
                    }
                });
            }
        });
    });
}

fn render_transaction(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label(format!(
            "{} Transaction for Account: {}",
            if app.working_transaction.id.is_some() {
                "Editing"
            } else {
                "Creating"
            },
            app.working_transaction.account_id
        ));
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            let label = ui.label("Amount: ".to_string());
            let mut text_amount = app.working_transaction.amount.clone().to_string();
            ui.text_edit_singleline(&mut text_amount)
                .labelled_by(label.id);
            if text_amount.parse::<f32>().is_ok() {
                app.working_transaction.amount = BigDecimal::from_str(&text_amount[..]).unwrap();
            }
        });
        ui.horizontal(|ui| {
            let label = ui.label("Type: ".to_string());
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", app.working_transaction.trx_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.working_transaction.trx_type,
                        TransactionType::Debit,
                        "Debit",
                    );
                    ui.selectable_value(
                        &mut app.working_transaction.trx_type,
                        TransactionType::Credit,
                        "Credit",
                    );
                })
                .response
                .labelled_by(label.id);
        });
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                app.state = DkkState::WalletView;
                let pool_ref = app.pool.clone();
                let ct_copy = app.working_transaction.clone();
                tokio::spawn(async move {
                    if ct_copy.upsert(&pool_ref).await.is_err() {
                        todo!("unhandled error");
                    }
                });
            }
        });
    });
}

fn render_wallet(ui: &mut egui::Ui, app: &mut Dkk) {
    app.wallet.get_mut(|wallet: Option<&mut Wallet>| {
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
                        wallet
                            .accounts
                            .sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

                        ui.label(RichText::new("Accounts").strong());
                        for acc in &wallet.accounts {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(format!("Id: {}", acc.id.unwrap()));
                                    ui.label(format!("Name: {}", acc.name));
                                    ui.label(format!("Type: {:?}", acc.acc_type));
                                    ui.label(format!(
                                        "Account Created date: {:?}",
                                        acc.created_date
                                    ));
                                    ui.label(format!("Balance: {:?}", get_account_balance(acc)));
                                    if ui.button("Edit account").clicked() {
                                        app.state = DkkState::AccountView;
                                        app.working_account = acc.clone();
                                    }
                                    for t in &acc.transactions {
                                        ui.group(|ui| {
                                            ui.label(format!("Transaction id: {}", t.id.unwrap()));
                                            ui.label(format!("Amount: {:?}", t.amount));
                                            ui.label(format!("Trx Type: {:?}", t.trx_type));
                                            if ui.button("Edit transaction").clicked() {
                                                app.state = DkkState::TransactionView;
                                                app.working_transaction = t.clone();
                                            }
                                        });
                                    }
                                    ui.group(|ui| {
                                        if ui.button("Create transaction").clicked() {
                                            app.state = DkkState::TransactionView;
                                            app.working_transaction = Transaction {
                                                account_id: acc.id.unwrap(),
                                                ..Default::default()
                                            };
                                        }
                                    });
                                });
                            });
                        }
                        if ui.button("Create account").clicked() {
                            app.state = DkkState::AccountView;
                            app.working_account = Account {
                                wallet_id: wallet.id,
                                ..Default::default()
                            };
                        }
                    });
                });
            });
        }
    });
}

fn handle_input(ui: &egui::Ui, app: &mut Dkk) {
    match app.state {
        DkkState::TransactionView | DkkState::AccountView => {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Escape) {
                    app.state = DkkState::WalletView;
                }
            });
        }
        DkkState::WalletView => {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Escape) {
                    app.state = DkkState::Init;
                    app.working_wallet = None;
                    let wallet_ref = app.wallet.clone();
                    if let Ok(mut guard) = wallet_ref.lock() {
                        *guard = None;
                    };
                }
            });
        }
        _ => {}
    }
}

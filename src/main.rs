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
    Wallet,
    CreateAccount,
    CreateTransaction,
    // @todo:edit-records: this is kind of related to the remove-structs todo since we could reuse
    // the DkkThreadData for each instead of creating new structs
}

struct Dkk {
    wallet: DkkThreadData<Wallet>,

    available_wallets: DkkThreadData<Vec<u32>>,
    selected_wallet: Option<u32>,

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
            selected_wallet: None,
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
                if let Ok(accounts) = get_wallet_accounts(wallet_id, &pool_ref).await {
                    for acc in accounts {
                        let ts = get_account_transactions(acc.id.unwrap(), &pool_ref).await;

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
        if let Some(wallet_id) = self.selected_wallet {
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
        DkkState::Wallet => render_wallet(ui, app),
        DkkState::CreateAccount => render_create_account(ui, app),
        DkkState::CreateTransaction => render_create_transaction(ui, app),
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
                            app.selected_wallet = Some(*wallet_id);
                            app.state = DkkState::Wallet;
                        }
                    }
                }
            });
        });
        ui.horizontal(|ui| {
            if ui.button("Create").clicked() {
                let pool_ref = app.pool.clone();
                tokio::spawn(async move {
                    match Wallet::create(&pool_ref).await {
                        Ok(_) => {}
                        Err(e) => {
                            todo!("unhandled error: {:?}", e)
                        }
                    }
                });
            }
        });
    });
}

fn render_create_account(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label(format!(
            "Creating account for wallet: {}",
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
            if ui.button("Create").clicked() {
                app.state = DkkState::Wallet;
                let pool_ref = app.pool.clone();
                let ca_copy = app.working_account.clone();
                tokio::spawn(async move {
                    match Account::create(
                        ca_copy.wallet_id,
                        ca_copy.name,
                        ca_copy.acc_type,
                        &pool_ref,
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            todo!("unhandled error: {:?}", e)
                        }
                    }
                });
            }
        });
    });
}

fn render_create_transaction(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label(format!(
            "Creating Transaction for account: {}",
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
            if ui.button("Create").clicked() {
                app.state = DkkState::Wallet;
                let pool_ref = app.pool.clone();
                let ct_copy = app.working_transaction.clone();
                tokio::spawn(async move {
                    match Transaction::create(
                        ct_copy.account_id,
                        f32::from_str(&ct_copy.amount.to_string()).unwrap(),
                        ct_copy.trx_type,
                        &pool_ref,
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            todo!("unhandled error: {:?}", e)
                        }
                    }
                });
            }
        });
    });
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
                        let mut accounts: Vec<&Account> = wallet.accounts.keys().collect();
                        accounts.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

                        ui.label(RichText::new("Accounts").strong());
                        for acc in accounts {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(format!("Id: {}", acc.id.unwrap()));
                                    ui.label(format!("Name: {}", acc.name));
                                    ui.label(format!("Type: {:?}", acc.acc_type));
                                    ui.label(format!(
                                        "Account Created date: {:?}",
                                        acc.created_date
                                    ));
                                    ui.label(format!(
                                        "Balance: {:?}",
                                        if let Some(transactions) = wallet.accounts.get(acc) {
                                            get_account_balance(&acc.acc_type, transactions)
                                        } else {
                                            0.0
                                        }
                                    ));
                                    if let Some(transactions) = wallet.accounts.get(acc) {
                                        for t in transactions {
                                            ui.group(|ui| {
                                                ui.label(format!(
                                                    "Transaction id: {}",
                                                    t.id.unwrap()
                                                ));
                                                ui.label(format!("Amount: {:?}", t.amount));
                                                ui.label(format!("Trx Type: {:?}", t.trx_type));
                                            });
                                        }
                                        ui.group(|ui| {
                                            if ui.button("Create transaction").clicked() {
                                                app.state = DkkState::CreateTransaction;
                                                app.working_transaction = Transaction {
                                                    account_id: acc.id.unwrap(),
                                                    ..Default::default()
                                                };
                                            }
                                        });
                                    }
                                });
                            });
                        }
                        if ui.button("Create account").clicked() {
                            app.state = DkkState::CreateAccount;
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
        DkkState::CreateTransaction | DkkState::CreateAccount => {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Escape) {
                    app.state = DkkState::Wallet;
                }
            });
        }
        DkkState::Wallet => {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Escape) {
                    app.state = DkkState::Init;
                    app.selected_wallet = None;
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

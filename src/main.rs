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
use models::{
    get_account_transactions, get_accounts_net_worth, get_wallet_accounts, Account, TransactionType,
};
use sqlx::{MySql, Pool};

use crate::models::Wallet;
use crate::settings::Settings;

enum DkkState {
    // @todo:init-state: There needs to be an initial state, perhaps to select a wallet or,
    // eventually, to perform a login operation Login/SelectWallet,

    // @todo:wallet-state: Keep the wallet struct within this enum?
    Wallet,
    CreateTransaction {
        account_id: u32,
        amount_str: String,
        trx_type: TransactionType,
    },
}

struct Dkk {
    // @todo:wallet-state: this dissappears
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
            match self.state {
                DkkState::Wallet => {
                    self.wallet.get(|wallet: Option<&Wallet>| {
                        if let Some(wallet) = wallet {
                            render_wallet(ui, wallet);
                        }
                    });
                }
                DkkState::CreateTransaction {
                    ref mut amount_str,
                    ref mut trx_type,
                    ref account_id,
                } => render_create_transaction(ui, amount_str, trx_type, account_id),
            };

            // handle input
            handle_input(ui, self);
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

fn render_create_transaction(
    ui: &mut egui::Ui,
    amount_str: &mut String,
    trx_type: &mut TransactionType,
    account_id: &u32,
) {
    ui.group(|ui| {
        ui.label(format!("Creating Transaction for account: {account_id}"));
    });

    ui.group(|ui| {
        ui.horizontal(|ui| {
            let label = ui.label("Amount: ".to_string());
            let prev_value = amount_str.clone();
            ui.text_edit_singleline(amount_str).labelled_by(label.id);
            if amount_str.parse::<f32>().is_err() {
                *amount_str = prev_value;
            }
        });
        ui.horizontal(|ui| {
            let label = ui.label("Type: ".to_string());
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", trx_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(trx_type, TransactionType::Debit, "Debit");
                    ui.selectable_value(trx_type, TransactionType::Credit, "Credit");
                })
                .response
                .labelled_by(label.id);
        });
        ui.horizontal(|ui| {
            // @todo:only-render-button: Perhaps an array of buttons within the Dkk Struct?
            // and that array gets passed to the handle_input
            // How do we identify each button to know which action to take? maybe a HashMap
            // instead?
            let _button = ui.button("Create");
        });
    });
}

fn render_wallet(ui: &mut egui::Ui, wallet: &Wallet) {
    ui.group(|ui| {
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
                                ui.label(format!("Account Created date: {:?}", acc.created_date));
                                if let Some(transactions) = wallet.accounts.get(acc) {
                                    for t in transactions {
                                        ui.group(|ui| {
                                            ui.label(format!("Transaction id: {}", t.id));
                                            ui.label(format!("Amount: {:?}", t.amount));
                                            ui.label(format!("Trx Type: {:?}", t.trx_type));
                                        });
                                    }
                                    ui.group(|ui| {
                                        let _button = ui.button("Create transaction");
                                        // @todo:only-render-button: render functions should not alter state.
                                        // how to keep button handle to manage it within
                                        // the handle_input fn

                                        // where will this be executed?

                                        //if button.clicked() {
                                        //    ui.label("Creating transaction");
                                        //    app.state = DkkState::CreateTransaction {
                                        //        account_id: acc.id,
                                        //        amount_str: String::new(),
                                        //        trx_type: TransactionType::Debit,
                                        //    }
                                        //}
                                    });
                                }
                            });
                        });
                    }
                });
            });
        });
    });
}

fn handle_input(ui: &egui::Ui, app: &mut Dkk) {
    if let DkkState::CreateTransaction { .. } = app.state {
        ui.input(|input| {
            if input.key_pressed(egui::Key::Escape) {
                app.state = DkkState::Wallet;
            }
        });
    };
}

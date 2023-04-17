mod alias;
mod error;
mod models;
mod settings;

use std::sync::Arc;
use std::time::Instant;

use alias::{DaikokuResult, DaikokuThreadData};
use eframe::egui;
use error::DaikokuError;
use models::{get_accounts_net_worth, get_wallet_accounts, Account};
use sqlx::{MySql, Pool};

use crate::models::Wallet;
use crate::settings::Settings;

struct Daikoku {
    wallet: DaikokuThreadData<Wallet>,
    accounts: DaikokuThreadData<Vec<Account>>,
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
            load_wallet(&self, wallet_id);
            load_accounts(&self, wallet_id);

            // render data
            self.wallet.get(|w: Option<&Wallet>| {
                if let Some(w) = w {
                    ui.label(format!("Wallet '{}'", w.id));
                    ui.label(format!("Created date '{:?}'", w.created_date));
                }
            });



            // see fps
            ctx.request_repaint();
            self.frame += 1;
            let sec_marker = self.start_time.elapsed().as_secs_f32() % 1.0;
            if sec_marker > 0.985 {
                self.fps = self.frame.clone();
                self.frame = 0;
            }
            ui.label(format!("fps '{:?}'", self.fps));
        });
    }
}

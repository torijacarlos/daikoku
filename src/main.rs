mod alias;
mod error;
mod models;
mod settings;

use alias::DaikokuResult;
use eframe::egui;
use error::DaikokuError;
use sqlx::{MySql, Pool};

use crate::models::{Account, AccountType, Transaction, TransactionType, Wallet};
use crate::settings::Settings;

struct Daikoku {
    wallet: Wallet,
}

#[tokio::main]
async fn main() -> DaikokuResult<()> {
    let settings = Settings::load().unwrap();
    let mut pool = settings.get_db_conn_pool().await?;
    let wallet = Wallet::create(&mut pool).await?;

    let app = Daikoku { wallet };

    Account::create(
        app.wallet.id,
        "test-asset".into(),
        AccountType::Asset,
        &mut pool,
    )
    .await?;
    Account::create(
        app.wallet.id,
        "test-liability".into(),
        AccountType::Liability,
        &mut pool,
    )
    .await?;
    Account::create(
        app.wallet.id,
        "test-equity".into(),
        AccountType::Equity,
        &mut pool,
    )
    .await?;

    for acc in app.wallet.get_accounts(&mut pool).await?.iter() {
        Transaction::create(acc.id, 1000.50, TransactionType::Debit, &mut pool).await?;
        Transaction::create(acc.id, 1000.50, TransactionType::Debit, &mut pool).await?;
        Transaction::create(acc.id, 500.25, TransactionType::Credit, &mut pool).await?;
    }

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native("My egui App", options, Box::new(|_cc| Box::new(app)))
        .map_err(DaikokuError::RenderError)
}

impl eframe::App for Daikoku {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Daikoku");
            ui.label(format!("Wallet '{}'", self.wallet.id));
        });
    }
}

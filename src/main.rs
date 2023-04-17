mod alias;
mod error;
mod models;
mod settings;

use std::sync::Arc;

use alias::{DaikokuResult, DaikokuThreadData};
use eframe::egui;
use error::DaikokuError;
use sqlx::{MySql, Pool};

use crate::models::Wallet;
use crate::settings::Settings;

struct Daikoku {
    wallet: DaikokuThreadData<Wallet>,
    pool: Arc<Pool<MySql>>,
    frame: u16,
}

impl Daikoku {
    fn new() -> Self {
        let settings = Settings::load().unwrap();
        let pool = Arc::new(settings.get_db_conn_pool());
        Self {
            wallet: DaikokuThreadData::empty(),
            pool,
            frame: 0,
        }
    }
}

#[tokio::main]
async fn main() -> DaikokuResult<()> {
    eframe::run_native(
        "Daikoku",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(Daikoku::new())),
    )
    .map_err(DaikokuError::RenderError)
}

impl eframe::App for Daikoku {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Daikoku");
            ui.label(format!("Frame '{}'", self.frame));

            // load data
            // render data
            self.wallet.get(|w: Option<&Wallet>| {
                if let Some(w) = w {
                    ui.label(format!("Wallet '{}'", w.id));
                }
            });
            self.frame += 1;
            self.frame %= 60;
        });
    }
}

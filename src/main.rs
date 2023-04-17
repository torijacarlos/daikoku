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
    settings: Arc<Settings>,
    pool: DaikokuThreadData<Pool<MySql>>,
    frame: u16,
}

impl Daikoku {
    fn new() -> Self {
        let settings = Arc::new(Settings::load().unwrap());
        Self {
            wallet: DaikokuThreadData::empty(),
            settings,
            pool: DaikokuThreadData::empty(),
            frame: 0,
        }
    }
}

fn prepare_database_pool(app: &Daikoku) {
    app.pool.get_option(|pool| {
        if let None = pool {
            let settings_ref = app.settings.clone();
            let pool_ref = app.pool.clone();
            tokio::spawn(async move {
                if let Ok(pool) = &settings_ref.get_db_conn_pool().await {
                    if let Ok(mut write_guard) = pool_ref.write() {
                        *write_guard = Some(pool.clone());
                    }
                }
            });
        }
    });
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
            // prepare application
            prepare_database_pool(&self);

            // load data
            // render data
            self.wallet.get(|w: &Wallet| {
                ui.label(format!("Wallet '{}'", w.id));
            });
            self.frame += 1;
        });
    }
}

mod alias;
mod error;
mod models;
mod settings;

use std::sync::Arc;

use alias::{DaikokuResult, DaikokuThreadData};
use eframe::egui;
use error::DaikokuError;

use crate::models::Wallet;
use crate::settings::Settings;

struct Daikoku {
    wallet: DaikokuThreadData<Wallet>,
    settings: Arc<Settings>,
    frame: u16,
}

impl Daikoku {
    fn new() -> Self {
        let settings = Arc::new(Settings::load().unwrap());
        Self {
            wallet: DaikokuThreadData::empty(),
            settings,
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
            // prepare application
            // @todo: prepare database pool
            // load data
            // render data
            self.wallet.get(|w: &Wallet| {
                ui.label(format!("Wallet '{}'", w.id));
            });
            self.frame += 1;
        });
    }
}

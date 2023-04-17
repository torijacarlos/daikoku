mod alias;
mod error;
mod models;
mod settings;

use std::sync::{Arc, Mutex};

use alias::DaikokuResult;
use eframe::egui;
use error::DaikokuError;

use crate::models::Wallet;
use crate::settings::Settings;

struct Daikoku {
    wallet: Arc<Mutex<Option<Result<Wallet, DaikokuError>>>>,
    settings: Arc<Settings>,
    frame: u16,
}

impl Daikoku {
    fn new(settings: Arc<Settings>) -> Self {
        let wallet = Arc::new(Mutex::new(None));
        let wallet_thread = wallet.clone();

        let set_ref = settings.clone();
        tokio::spawn(async move {
            if let Ok(ref pool) = &set_ref.get_db_conn_pool().await {
                let result = Wallet::get(1, pool).await;
                if let Ok(mut mutex_lock) = wallet_thread.lock() {
                    *mutex_lock = Some(result);
                }
            }
        });

        Self {
            wallet,
            settings,
            frame: 0,
        }
    }
}

#[tokio::main]
async fn main() -> DaikokuResult<()> {
    let settings = Arc::new(Settings::load().unwrap());

    let app = Daikoku::new(settings);

    eframe::run_native(
        "Daikoku",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(app)),
    )
    .map_err(DaikokuError::RenderError)
}

impl eframe::App for Daikoku {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Daikoku");
            ui.label(format!("Frame '{}'", self.frame));

            if let Ok(mut wallet_guard) = self.wallet.try_lock() {
                match &mut *wallet_guard {
                    Some(Ok(ref w)) => {
                        ui.label(format!("Wallet '{}'", w.id));
                    }
                    _ => {}
                };
            }
            self.frame += 1;
        });
    }
}

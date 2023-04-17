mod alias;
mod error;
mod models;
mod settings;

use std::sync::{Arc, Mutex};

use alias::{DaikokuResult, ThreadData};
use eframe::egui;
use error::DaikokuError;

use crate::models::Wallet;
use crate::settings::Settings;




struct Daikoku {
    // (torijacarlos:todo) after all pending todos - retry the struct DaikokuThreadData<T>(ThreadData<T>)
    //                     remember to add the Send and Sync traits to it!!!!!!!
    //                     you need to implement 
    //                          impl Default for DaikokuThreadData
    //                          impl<T> DaikokuThreadData<T> { fn new(v: T) }
    wallet: ThreadData<Wallet>,
    settings: Arc<Settings>,
    frame: u16,
}

impl Daikoku {
    fn new() -> Self {
        Self {
            wallet: Arc::new(Mutex::new(None)),
            settings: Arc::new(Settings::load().unwrap()),
            frame: 0,
        }
    }

    fn load_wallet(&self) {
        let wallet_ref = self.wallet.clone();
        let set_ref = self.settings.clone();

        tokio::spawn(async move {
            if let Ok(ref pool) = &set_ref.get_db_conn_pool().await {
                let result = Wallet::get(1, pool).await;
                if let Ok(mut mutex_lock) = wallet_ref.lock() {
                    *mutex_lock = Some(result);
                }
            }
        });

    }
}

#[tokio::main]
async fn main() -> DaikokuResult<()> {

    let app = Daikoku::new();

    eframe::run_native(
        "Daikoku",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(app)),
    )
    .map_err(DaikokuError::RenderError)
}

// (torijacarlos:todo) explore available ui elements
impl eframe::App for Daikoku {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Daikoku");
            ui.label(format!("Frame '{}'", self.frame));

            self.load_wallet();
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

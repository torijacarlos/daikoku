pub mod alias;
pub mod error;
mod models;
mod settings;
mod storage;
mod ui;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use alias::DkkThreadData;
use egui::Align;
use models::{Account, Transaction, Wallet};
use settings::Settings;
use sqlx::{MySql, Pool};
use ui::{handle_input, render, DkkUiState};

pub struct Dkk {
    pub pin: String,
    pub wallet: DkkThreadData<Wallet>,

    pub available_wallets: DkkThreadData<Vec<u32>>,

    pub working_wallet: Option<u32>,
    pub working_account: Account,
    pub working_transaction: Transaction,

    pub crypt_key: String,
    pub pool: Arc<Pool<MySql>>,

    pub force_reload: bool,
    pub fps: f32,
    pub frame: u128,
    pub frame_time: Instant,
    pub state: DkkUiState,
}

impl Dkk {
    pub fn new() -> Self {
        let settings = Settings::load().unwrap();
        Self {
            pin: String::new(),
            wallet: DkkThreadData::empty(),
            state: DkkUiState::Init,
            available_wallets: DkkThreadData::empty(),
            working_wallet: None,
            working_account: Account::default(),
            working_transaction: Transaction::default(),
            crypt_key: settings.crypt_key.clone(),
            pool: Arc::new(settings.get_db_conn_pool()),
            force_reload: false,
            fps: 0.0,
            frame: 0,
            frame_time: Instant::now(),
        }
    }
}

impl eframe::App for Dkk {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |gui| {
            gui.with_layout(
                egui::Layout::top_down_justified(Align::LEFT).with_cross_justify(true),
                |gui| {
                    egui::ScrollArea::vertical()
                        .id_source("first")
                        .show(gui, |gui| {
                            render(gui, self);
                        });
                },
            );
            handle_input(gui, self);
        });
        storage::load(self);
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

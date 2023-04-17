mod render_account;
mod render_init;
mod render_transaction;
mod render_wallet;

use crate::{Dkk, storage::get_all_wallets_locations};

pub enum DkkUiState {
    Init,
    WalletView,
    TransactionView,
}

pub fn render(gui: &mut egui::Ui, app: &mut Dkk) {
    gui.horizontal_top(|gui| {
        gui.heading("Dkk");
        gui.label(format!("fps: {:?}", app.fps));
    });
    match app.state {
        DkkUiState::Init => render_init::render_init(gui, app),
        DkkUiState::WalletView => render_wallet::render_wallet(gui, app),
        DkkUiState::TransactionView => render_transaction::render_transaction(gui, app),
    };
}

pub fn handle_input(ui: &egui::Ui, app: &mut Dkk) {
    match app.state {
        DkkUiState::TransactionView => {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Escape) {
                    app.state = DkkUiState::WalletView;
                }
            });
        }
        DkkUiState::WalletView => {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Escape) {
                    app.working_account_id = None;
                    app.available_wallets = get_all_wallets_locations();
                }
            });
        }
        _ => {}
    }
}

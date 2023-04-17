mod render_account;
mod render_init;
mod render_transaction;
mod render_wallet;

use crate::Dkk;

pub enum DkkUiState {
    Init,
    WalletView,
    AccountView,
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
        DkkUiState::AccountView => render_account::render_account(gui, app),
        DkkUiState::TransactionView => render_transaction::render_transaction(gui, app),
    };
}

pub fn handle_input(ui: &egui::Ui, app: &mut Dkk) {
    match app.state {
        DkkUiState::TransactionView | DkkUiState::AccountView => {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Escape) {
                    app.state = DkkUiState::WalletView;
                }
            });
        }
        DkkUiState::WalletView => {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Escape) {
                    app.state = DkkUiState::Init;
                    app.working_wallet = None;
                    app.working_alias = String::new();
                    let wallet_ref = app.wallet.clone();
                    if let Ok(mut guard) = wallet_ref.lock() {
                        *guard = None;
                    };
                }
            });
        }
        _ => {}
    }
}

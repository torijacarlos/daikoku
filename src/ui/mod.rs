mod render_account;
mod render_init;
mod render_transaction;
mod render_wallet;

use crate::{storage::get_all_wallets_locations, Dkk};

pub enum DkkUiState {
    Init,
    WalletView,
    // @todo: This states usage and values will dissappear, but i'll still keep an state
    // the usage would be, for example, avoid the boolean param in render_account (editing)
    // and instead use the state. in that case the State could have an struct
    // AccountEdition { id: Uuid }
    // Same for Transaction and Wallet
    // the current render function will most likely dissappear as it is
}

pub fn render(gui: &mut egui::Ui, app: &mut Dkk) {
    gui.horizontal_top(|gui| {
        gui.heading("Dkk");
        gui.label(format!("fps: {:?}", app.fps));
    });
    match app.state {
        DkkUiState::Init => render_init::render_init(gui, app),
        DkkUiState::WalletView => render_wallet::render_wallet(gui, app),
    };
}

pub fn handle_input(ui: &egui::Ui, app: &mut Dkk) {
    match app.state {
        DkkUiState::WalletView => {
            ui.input(|input| {
                if input.key_pressed(egui::Key::Escape) {
                    app.working_account_id = None;
                    app.working_transaction_id = None;
                    app.available_wallets = get_all_wallets_locations();
                }
            });
        }
        _ => {}
    }
}

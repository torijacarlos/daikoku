mod render_account;
mod render_transaction;
mod render_wallet;

use crate::{
    models::Wallet,
    storage::{self, get_all_wallets_locations},
    Dkk,
};

// @todo: The DkkUiState dissappeared, but I think there was value in it.
// for example, avoid the boolean param in render_account (editing)
// and instead use the state. in that case the State could have an struct
// WalletView { wallet: Wallet }
// AccountEdition { id: Uuid }
// TransactionEdition { id: Uuid }

pub fn render(gui: &mut egui::Ui, app: &mut Dkk) {
    gui.horizontal_top(|gui| {
        gui.heading("Dkk");
        gui.label(format!("fps: {:?}", app.fps));
    });
    gui.group(|ui| {
        ui.label("Select or create a wallet");
        ui.vertical(|ui| {
            let label = ui.label("Pin: ".to_string());
            ui.text_edit_singleline(&mut app.pin).labelled_by(label.id);
            for aw in &app.available_wallets {
                if ui.button(format!("{:?}", aw)).clicked() {
                    if let Ok(wallet) = storage::import(aw.to_path_buf(), &app.pin, &app.crypt_key)
                    {
                        app.wallet = Some(wallet);
                    }
                }
            }
        });
        ui.horizontal(|ui| {
            let label = ui.label("Alias: ".to_string());
            ui.text_edit_singleline(&mut app.working_alias)
                .labelled_by(label.id);
            if ui.button("Create").clicked() && app.working_alias.len() > 2 {
                app.wallet = Some(Wallet {
                    alias: app.working_alias.to_string(),
                    ..Default::default()
                });
            }
        });
        ui.horizontal(|ui| {
            if app.wallet.is_some() {
                render_wallet::render_wallet(ui, app)
            }
        })
    });
}

pub fn handle_input(ui: &egui::Ui, app: &mut Dkk) {
    ui.input(|input| {
        if input.key_pressed(egui::Key::Escape) {
            if app.working_transaction_id.is_none() && app.working_account_id.is_none() {
                app.wallet = None;
                app.pin = String::new();
            } else {
                app.working_account_id = None;
                app.working_transaction_id = None;
                app.available_wallets = get_all_wallets_locations();
            }
        }
    });
}

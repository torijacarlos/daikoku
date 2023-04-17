use crate::{models::Wallet, storage, Dkk};

use super::DkkUiState;

pub fn render_init(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label("Select or create a wallet");
        ui.vertical(|ui| {
            let label = ui.label("Pin: ".to_string());
            ui.text_edit_singleline(&mut app.pin).labelled_by(label.id);
            for aw in &app.available_wallets {
                if ui.button(format!("{:?}", aw)).clicked() {
                    if let Ok(wallet) = storage::import(aw.to_path_buf(), &app.pin, &app.crypt_key)
                    {
                        app.wallet = Some(wallet);
                        app.state = DkkUiState::WalletView;
                    }
                }
            }
        });
        ui.horizontal(|ui| {
            let label = ui.label("Alias: ".to_string());
            ui.text_edit_singleline(&mut app.working_alias)
                .labelled_by(label.id);
            if ui.button("Create").clicked() && app.working_alias.len() > 2 {
                app.state = DkkUiState::WalletView;
                app.wallet = Some(Wallet {
                    alias: app.working_alias.to_string(),
                    ..Default::default()
                });
            }
        });
    });
}

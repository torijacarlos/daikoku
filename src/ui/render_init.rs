use crate::{models::Wallet, storage, Dkk};

use super::DkkUiState;

pub fn render_init(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label("Select or create a wallet");
        ui.vertical(|ui| {
            app.available_wallets.get(|aw| {
                if let Some(aw) = aw {
                    for wallet in aw {
                        if ui.button(format!("{} {}", wallet.id.unwrap(), wallet.alias)).clicked() {
                            app.working_wallet = Some(wallet.id.unwrap());
                            app.state = DkkUiState::WalletView;
                        }
                    }
                }
            });
        });
        ui.horizontal(|ui| {
            let pool_ref = app.pool.clone();
            ui.horizontal(|ui| {
                let label = ui.label("Pin: ".to_string());
                ui.text_edit_singleline(&mut app.pin).labelled_by(label.id);
                if ui.button("Import").clicked() {
                    let pin = app.pin.clone();
                    let key = app.crypt_key.clone();
                    tokio::spawn(async move {
                        println!("Importing");
                        let _ = storage::import(&pool_ref, &pin, &key).await;
                    });
                }
            });
        });
        ui.horizontal(|ui| {
            let label = ui.label("Alias: ".to_string());
            ui.text_edit_singleline(&mut app.working_alias)
                .labelled_by(label.id);
            if ui.button("Create").clicked() {
                let pool_ref = app.pool.clone();
                let alias = app.working_alias.clone();
                tokio::spawn(async move {
                    let mut wallet = Wallet::default();
                    wallet.alias = alias;
                    if wallet.upsert(&pool_ref).await.is_err() {
                        todo!("unhandled error");
                    }
                });
            }
        });
    });
}

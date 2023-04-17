use crate::{models::Wallet, storage, Dkk};

use super::DkkUiState;

pub fn render_init(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label("Select or create a wallet");
        ui.vertical(|ui| {
            app.available_wallets.get(|aw| {
                if let Some(aw) = aw {
                    for wallet_id in aw {
                        if ui.button(format!("{wallet_id}")).clicked() {
                            app.working_wallet = Some(*wallet_id);
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
            if ui.button("Create").clicked() {
                let pool_ref = app.pool.clone();
                tokio::spawn(async move {
                    let mut wallet = Wallet::default();
                    if wallet.upsert(&pool_ref).await.is_err() {
                        todo!("unhandled error");
                    }
                });
            }
        });
    });
}

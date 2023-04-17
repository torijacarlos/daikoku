use crate::{models::Wallet, Dkk, storage};

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
            if ui.button("Import").clicked() {
                let pool_ref = app.pool.clone();
                tokio::spawn(async move {
                    storage::import(&pool_ref).await;
                });
            }
            if ui.button("Create").clicked() {
                let pool_ref = app.pool.clone();
                tokio::spawn(async move {
                    let wallet = Wallet::default();
                    if wallet.upsert(&pool_ref).await.is_err() {
                        todo!("unhandled error");
                    }
                });
            }
        });
    });
}

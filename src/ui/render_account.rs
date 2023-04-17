use crate::{models::AccountType, Dkk};

use super::DkkUiState;

pub fn render_account(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label(format!(
            "{} Account for Wallet: {}",
            if app.working_account.id.is_some() {
                "Editing"
            } else {
                "Creating"
            },
            app.working_account.wallet_id
        ));
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            let label = ui.label("Name: ".to_string());
            ui.text_edit_singleline(&mut app.working_account.name)
                .labelled_by(label.id);
        });
        ui.horizontal(|ui| {
            let label = ui.label("Type: ".to_string());
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", app.working_account.acc_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Asset,
                        "Asset",
                    );
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Equity,
                        "Equity",
                    );
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Expense,
                        "Expense",
                    );
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Income,
                        "Income",
                    );
                    ui.selectable_value(
                        &mut app.working_account.acc_type,
                        AccountType::Liability,
                        "Liability",
                    );
                })
                .response
                .labelled_by(label.id);
        });
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                app.state = DkkUiState::WalletView;
                let pool_ref = app.pool.clone();
                let ca_copy = app.working_account.clone();
                tokio::spawn(async move {
                    if ca_copy.upsert(&pool_ref).await.is_err() {
                        todo!("unhandled error");
                    }
                });
            }
        });
    });
}

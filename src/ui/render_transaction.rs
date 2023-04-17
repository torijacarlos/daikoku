use std::str::FromStr;

use chrono::{NaiveTime, TimeZone, Utc};
use sqlx::types::BigDecimal;

use crate::{models::TransactionType, Dkk};

use super::DkkUiState;

pub fn render_transaction(ui: &mut egui::Ui, app: &mut Dkk) {
    ui.group(|ui| {
        ui.label(format!(
            "{} Transaction for Account: {}",
            if app.working_transaction.id.is_some() {
                "Editing"
            } else {
                "Creating"
            },
            app.working_transaction.account_id
        ));
    });
    ui.group(|ui| {
        ui.horizontal(|ui| {
            let label = ui.label("Amount: ".to_string());
            let mut text_amount = app.working_transaction.amount.clone().to_string();
            ui.text_edit_singleline(&mut text_amount)
                .labelled_by(label.id);
            if text_amount.parse::<f32>().is_ok() {
                app.working_transaction.amount = BigDecimal::from_str(&text_amount[..]).unwrap();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Execution Date: ".to_string());
            let mut date_picker = app.working_transaction.execution_date.naive_utc().date();
            ui.add(egui_extras::DatePickerButton::new(&mut date_picker));
            app.working_transaction.execution_date = Utc.from_utc_datetime(
                &date_picker.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
            );
        });
        ui.horizontal(|ui| {
            let label = ui.label("Type: ".to_string());
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", app.working_transaction.trx_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.working_transaction.trx_type,
                        TransactionType::Debit,
                        "Debit",
                    );
                    ui.selectable_value(
                        &mut app.working_transaction.trx_type,
                        TransactionType::Credit,
                        "Credit",
                    );
                })
                .response
                .labelled_by(label.id);
        });
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                app.state = DkkUiState::WalletView;
                let pool_ref = app.pool.clone();
                let mut ct_copy = app.working_transaction.clone();
                tokio::spawn(async move {
                    if ct_copy.upsert(&pool_ref).await.is_err() {
                        todo!("unhandled error");
                    }
                });
            }
        });
    });
}

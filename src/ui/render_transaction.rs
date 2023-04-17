use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::{NaiveTime, TimeZone, Utc};

use crate::models::{Transaction, TransactionType};

pub fn render_transaction(ui: &mut egui::Ui, transaction: &mut Transaction, editing: bool) {
    if editing {
        edit_transaction(ui, transaction);
    } else {
        view_transaction(ui, transaction);
    }
}

fn view_transaction(ui: &mut egui::Ui, transaction: &mut Transaction) {
    ui.label(format!("Amount: {:?}", transaction.amount));
    ui.label(format!("Date: {:?}", transaction.execution_date));
    ui.label(format!("Trx Type: {:?}", transaction.trx_type));
}

fn edit_transaction(ui: &mut egui::Ui, transaction: &mut Transaction) {
    ui.group(|ui| {
        ui.label(format!("Id: {:?}", transaction.id));
        ui.horizontal(|ui| {
            let label = ui.label("Amount: ".to_string());
            let mut text_amount = transaction.amount.clone().to_string();
            ui.text_edit_singleline(&mut text_amount)
                .labelled_by(label.id);
            if text_amount.parse::<f32>().is_ok() {
                transaction.amount = BigDecimal::from_str(&text_amount[..]).unwrap();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Execution Date: ".to_string());
            let mut date_picker = transaction.execution_date.naive_utc().date();
            ui.add(egui_extras::DatePickerButton::new(&mut date_picker));
            transaction.execution_date = Utc.from_utc_datetime(
                &date_picker.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
            );
        });
        ui.horizontal(|ui| {
            let label = ui.label("Type: ".to_string());
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", transaction.trx_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut transaction.trx_type, TransactionType::Debit, "Debit");
                    ui.selectable_value(
                        &mut transaction.trx_type,
                        TransactionType::Credit,
                        "Credit",
                    );
                })
                .response
                .labelled_by(label.id);
        });
    });
}

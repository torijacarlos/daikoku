use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::{NaiveTime, TimeZone, Utc};

use crate::{models::AccountType, Dkk};

use super::DkkUiState;

pub fn render_account(ui: &mut egui::Ui, app: &mut Dkk) {
    if let Some(ref mut account) = app.working_account {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let label = ui.label("Name: ".to_string());
                ui.text_edit_singleline(&mut account.name)
                    .labelled_by(label.id);
            });
            ui.horizontal(|ui| {
                let label = ui.label("Balance: ".to_string());
                let mut text_amount = account.balance.clone().to_string();
                ui.text_edit_singleline(&mut text_amount)
                    .labelled_by(label.id);
                if text_amount.parse::<f32>().is_ok() {
                    account.balance = BigDecimal::from_str(&text_amount[..]).unwrap();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Balance Date: ".to_string());
                let mut date_picker = account.balance_date.naive_utc().date();
                ui.add(egui_extras::DatePickerButton::new(&mut date_picker));
                account.balance_date = Utc.from_utc_datetime(
                    &date_picker.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                );
            });
            ui.horizontal(|ui| {
                let label = ui.label("Type: ".to_string());
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", account.acc_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut account.acc_type, AccountType::Asset, "Asset");
                        ui.selectable_value(&mut account.acc_type, AccountType::Equity, "Equity");
                        ui.selectable_value(&mut account.acc_type, AccountType::Expense, "Expense");
                        ui.selectable_value(&mut account.acc_type, AccountType::Income, "Income");
                        ui.selectable_value(
                            &mut account.acc_type,
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
                    // @todo: push account to wallet
                    // account
                }
            });
        });
    }
}

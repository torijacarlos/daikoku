use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::{NaiveTime, TimeZone, Utc};

use crate::models::{Account, AccountType, get_account_balance};

pub fn render_account(ui: &mut egui::Ui, account: &mut Account, editing: bool) {
    if editing {
        editing_account(ui, account);
    } else {
        viewing_account(ui, account);
    }
}

fn viewing_account(ui: &mut egui::Ui, account: &mut Account) {
    ui.label(format!("Id: {:?}", account.id));
    ui.label(format!("Name: {}", account.name));
    ui.label(format!("Type: {:?}", account.acc_type));
    ui.label(format!("Balance: {:?}", account.balance));
    ui.label(format!("Current Balance: {:?}", get_account_balance(account)));
    ui.label(format!("Balance date: {}", account.balance_date));
    ui.label(format!("Created date: {}", account.created_date));
}

fn editing_account(ui: &mut egui::Ui, account: &mut Account) {
    ui.group(|ui| {
        ui.label(format!("Id: {:?}", account.id));
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
                    ui.selectable_value(&mut account.acc_type, AccountType::Liability, "Liability");
                })
                .response
                .labelled_by(label.id);
        });
    });
}

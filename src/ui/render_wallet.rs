use egui::RichText;

use crate::{
    models::{get_accounts_net_worth, get_wallet_liquidity_index, Account},
    storage, Dkk,
};

use super::{render_account::render_account, DkkUiState};

pub fn render_wallet(ui: &mut egui::Ui, app: &mut Dkk) {
    if let Some(ref mut wallet) = app.wallet {
        ui.vertical(|ui| {
            ui.label(RichText::new("Wallet information").strong());
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let label = ui.label("Pin: ".to_string());
                        ui.text_edit_singleline(&mut app.pin).labelled_by(label.id);
                        if ui.button("Export Wallet to file").clicked() {
                            storage::export(wallet, &app.pin, &app.crypt_key);
                        }
                    });
                });
                ui.group(|ui| {
                    ui.label(format!("Id: {:?}", wallet.id));
                    ui.label(format!("Alias: {}", wallet.alias));
                    ui.label(format!("Created date: {:?}", wallet.created_date));
                    ui.label(format!(
                        "Net Worth: {:?}",
                        get_accounts_net_worth(&wallet.accounts)
                    ));
                    ui.label(format!(
                        "Liquidity Index: {:?}",
                        get_wallet_liquidity_index(&wallet.accounts)
                    ));
                });
                ui.vertical(|ui| {
                    wallet
                        .accounts
                        .sort_by(|a, b| a.created_date.partial_cmp(&b.created_date).unwrap());

                    ui.label(RichText::new("Accounts").strong());
                    if ui.button("Create account").clicked() {
                        let acc = Account::new();
                        wallet.accounts.push(acc.clone());
                        app.working_account_id = acc.id;
                    }
                    for acc in wallet.accounts.iter_mut() {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                if ui.button("Edit account").clicked() {
                                    app.working_account_id = acc.id;
                                }
                                render_account(ui, acc, acc.id == app.working_account_id);

                                for t in &acc.transactions {
                                    ui.group(|ui| {
                                        ui.label(format!("Amount: {:?}", t.amount));
                                        ui.label(format!("Date: {:?}", t.execution_date));
                                        ui.label(format!("Trx Type: {:?}", t.trx_type));
                                        if ui.button("Edit transaction").clicked() {
                                            app.state = DkkUiState::TransactionView;
                                            // @todo: mutable reference to transaction
                                        }
                                    });
                                }
                                ui.group(|ui| {
                                    if ui.button("Create transaction").clicked() {
                                        app.state = DkkUiState::TransactionView;
                                    }
                                });
                            });
                        });
                    }
                });
            });
        });
    }
}

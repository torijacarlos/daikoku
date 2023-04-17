use egui::RichText;

use crate::{
    models::{
        get_account_balance, get_accounts_net_worth, get_wallet_liquidity_index, Account,
        Transaction, Wallet,
    },
    Dkk,
};

use super::DkkUiState;

pub fn render_wallet(ui: &mut egui::Ui, app: &mut Dkk) {
    app.wallet.get_mut(|wallet: Option<&mut Wallet>| {
        if let Some(wallet) = wallet {
            ui.vertical(|ui| {
                ui.label(RichText::new("Wallet information").strong());
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label(format!("Id: {}", wallet.id));
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
                            .sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

                        ui.label(RichText::new("Accounts").strong());
                        if ui.button("Create account").clicked() {
                            app.state = DkkUiState::AccountView;
                            app.working_account = Account {
                                wallet_id: wallet.id,
                                ..Default::default()
                            };
                        }
                        for acc in &wallet.accounts {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(format!("Id: {}", acc.id.unwrap()));
                                    ui.label(format!("Name: {}", acc.name));
                                    ui.label(format!("Type: {:?}", acc.acc_type));
                                    ui.label(format!("Balance date: {}", acc.balance_date));
                                    ui.label(format!(
                                        "Created date: {}",
                                        acc.created_date.unwrap()
                                    ));
                                    ui.label(format!("Balance: {:?}", get_account_balance(acc)));
                                    if ui.button("Edit account").clicked() {
                                        app.state = DkkUiState::AccountView;
                                        app.working_account = acc.clone();
                                    }
                                    for t in &acc.transactions {
                                        ui.group(|ui| {
                                            ui.label(format!("Transaction id: {}", t.id.unwrap()));
                                            ui.label(format!("Amount: {:?}", t.amount));
                                            ui.label(format!("Trx Type: {:?}", t.trx_type));
                                            if ui.button("Edit transaction").clicked() {
                                                app.state = DkkUiState::TransactionView;
                                                app.working_transaction = t.clone();
                                            }
                                        });
                                    }
                                    ui.group(|ui| {
                                        if ui.button("Create transaction").clicked() {
                                            app.state = DkkUiState::TransactionView;
                                            app.working_transaction = Transaction {
                                                account_id: acc.id.unwrap(),
                                                ..Default::default()
                                            };
                                        }
                                    });
                                });
                            });
                        }
                    });
                });
            });
        }
    });
}

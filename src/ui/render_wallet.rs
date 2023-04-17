use egui::RichText;

use crate::{
    models::{get_accounts_net_worth, get_wallet_liquidity_index, Account, Transaction},
    Dkk,
};

use super::{render_account::render_account, render_transaction::render_transaction};

pub fn render_wallet(ui: &mut egui::Ui, app: &mut Dkk) {
    if let Some(ref mut wallet) = app.wallet {
        ui.vertical(|ui| {
            ui.label(RichText::new("Wallet information").strong());
            ui.vertical(|ui| {
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
                                // @todo: show them as cards, meaning, all account that fit on a
                                // row, instead of 1 every row
                                // If not, change them to a table 
                                render_account(ui, acc, acc.id == app.working_account_id);

                                for t in acc.transactions.iter_mut() {
                                    ui.group(|ui| {
                                        if ui.button("Edit transaction").clicked() {
                                            app.working_transaction_id = t.id;
                                        }
                                        // @todo: egui has a table. change transactions to that
                                        // maybe a floating?
                                        // I think this makes them a floating window
                                        // https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/widget_gallery.rs#L53
                                        render_transaction(
                                            ui,
                                            t,
                                            t.id == app.working_transaction_id,
                                        );
                                    });
                                }
                                ui.group(|ui| {
                                    if ui.button("Create transaction").clicked() {
                                        let t = Transaction::new();
                                        acc.transactions.push(t.clone());
                                        app.working_transaction_id = t.id;
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

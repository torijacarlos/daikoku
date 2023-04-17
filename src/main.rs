mod models;
mod settings;

use crate::models::{Account, AccountType, Transaction, TransactionType, Wallet};
use crate::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let settings = Settings::load().unwrap();
    let mut pool = settings.get_db_conn_pool().await?;

    let wallet = Wallet::create(&mut pool).await?;

    println!("{:#?}", wallet);

    let acc_asset = Account::create(wallet.id, "test-asset".into(), AccountType::Asset, &mut pool).await;
    let acc_liability = Account::create(wallet.id, "test-liability".into(), AccountType::Liability, &mut pool).await;
    let acc_equity = Account::create(wallet.id, "test-liability".into(), AccountType::Equity, &mut pool).await;

    println!("{:#?}", acc_asset);
    println!("{:#?}", acc_liability);
    println!("{:#?}", acc_equity);

    for acc in wallet.get_accounts(&mut pool).iter_mut() {
        Transaction::create(acc.id, 1000.0, TransactionType::Debit, &mut pool);
        Transaction::create(acc.id, 1000.0, TransactionType::Debit, &mut pool);
        Transaction::create(acc.id, 500.0, TransactionType::Credit, &mut pool);
        println!("{}: {}", acc.name, acc.balance(&mut pool));
    }
    println!("Wallet: {}", wallet.net_worth(&mut pool));

    Ok(())
}

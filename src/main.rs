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

    let _ = Account::create(wallet.id, "test-asset".into(), AccountType::Asset);
    let _ = Account::create(wallet.id, "test-liability".into(), AccountType::Liability);

    for acc in wallet.get_accounts(&mut pool).iter_mut() {
        acc.transactions
            .push(Transaction::new(0, 1000.0, TransactionType::Debit));
        acc.transactions
            .push(Transaction::new(0, 1000.0, TransactionType::Debit));
        acc.transactions
            .push(Transaction::new(0, 500.0, TransactionType::Credit));
        println!("{}: {}", acc.name, acc.balance());
    }
    println!("Wallet: {}", wallet.net_worth(&mut pool));

    Ok(())
}

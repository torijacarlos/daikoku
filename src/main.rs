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

    Account::create(
        wallet.id,
        "test-asset".into(),
        AccountType::Asset,
        &mut pool,
    )
    .await?;
    Account::create(
        wallet.id,
        "test-liability".into(),
        AccountType::Liability,
        &mut pool,
    )
    .await?;
    Account::create(
        wallet.id,
        "test-equity".into(),
        AccountType::Equity,
        &mut pool,
    )
    .await?;

    for acc in wallet.get_accounts(&mut pool).await?.iter() {
        println!("{:#?}", acc);
        Transaction::create(acc.id, 1000.50, TransactionType::Debit, &mut pool).await?;
        Transaction::create(acc.id, 1000.50, TransactionType::Debit, &mut pool).await?;
        Transaction::create(acc.id, 500.25, TransactionType::Credit, &mut pool).await?;
        println!("{}: {}", acc.name, acc.balance(&mut pool).await?);
    }
    println!("Wallet: {}", wallet.net_worth(&mut pool).await?);

    Ok(())
}

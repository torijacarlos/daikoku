mod models;
mod settings;

use crate::models::{Account, AccountType, Transaction, TransactionType, Wallet};
use crate::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let settings = Settings::load().unwrap();
    let mut conn = settings.get_db_conn().await?;

    let _results =
        sqlx::query!(r#"SELECT id, username FROM WALLET where username = "torijacarlos""#)
            .fetch_optional(&mut conn)
            .await?;

    let mut wallet = Wallet::new("torijacarlos");
    //wallet.save();
    //println!("{:#?}", wallet.id);
    wallet.accounts = vec![
        Account::new(0, "test-asset", AccountType::Asset),
        Account::new(0, "test-liability", AccountType::Liability),
    ];

    println!("{}", u32::MAX);
    for acc in wallet.accounts.iter_mut() {
        acc.transactions
            .push(Transaction::new(0, 1000.0, TransactionType::Debit));
        acc.transactions
            .push(Transaction::new(0, 1000.0, TransactionType::Debit));
        acc.transactions
            .push(Transaction::new(0, 500.0, TransactionType::Credit));
        println!("{}: {}", acc.name, acc.balance());
    }
    println!("Wallet: {}", wallet.net_worth());

    Ok(())
}

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
enum AccountType {
    Asset,
    Liability,
    Expense,
    Income,
    Equity,
}

#[derive(Debug)]
enum TransactionType {
    Debit,
    Credit,
}

trait Storage {
    fn get(id: u32) -> Self;
    fn save();
}

struct Wallet<'a> {
    id: Option<u32>,
    username: &'a str,
    accounts: Vec<Account<'a>>,
}

impl<'a> Wallet<'a> {
    fn new(username: &'a str) -> Self {
        Self {
            id: None,
            username,
            accounts: vec![],
        }
    }

    fn net_worth(&self) -> f32 {
        let mut total = 0.0;
        for acc in &self.accounts {
            total += acc.balance();
        }
        total
    }
}

impl<'a> Storage for Wallet<'a> {
    fn get(id: u32) -> Self {
        todo!();
    }

    fn save() {
        todo!();
    }
}

#[derive(Debug)]
struct Account<'a> {
    id: Option<u32>,
    wallet_id: u32,
    name: &'a str,
    acc_type: AccountType,
    created_date: DateTime<Utc>,
    transactions: Vec<Transaction>,
}

impl<'a> Account<'a> {
    fn new(wallet_id: u32, name: &'a str, acc_type: AccountType) -> Self {
        Self {
            id: None,
            name,
            acc_type,
            wallet_id,
            created_date: Utc::now(),
            transactions: vec![],
        }
    }

    fn balance(&self) -> f32 {
        let mut total: f32 = 0.0;
        let multiplier = match &self.acc_type {
            AccountType::Asset | AccountType::Expense => 1.0,
            _ => -1.0,
        };
        for trx in &self.transactions {
            match trx.trx_type {
                TransactionType::Debit => total += trx.amount * multiplier,
                TransactionType::Credit => total -= trx.amount * multiplier,
            }
        }
        total
    }
}

#[derive(Debug)]
struct Transaction {
    id: Option<u32>,
    amount: f32,
    execution_date: DateTime<Utc>,
    trx_type: TransactionType,
    account_id: u32,
}

impl Transaction {
    fn new(account_id: u32, amount: f32, trx_type: TransactionType) -> Self {
        Transaction {
            id: None,
            amount,
            account_id,
            trx_type,
            execution_date: Utc::now(),
        }
    }
}

fn main() {
    let mut wallet = Wallet::new("torijacarlos");
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
}

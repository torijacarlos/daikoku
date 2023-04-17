use chrono::{DateTime, Utc};

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

struct Wallet<'a> {
    accounts: Vec<Account<'a>>,
}

impl<'a> Wallet<'a> {
    fn net_worth(&self) -> f32 {
        let mut total = 0.0;
        for acc in &self.accounts {
            total += acc.balance();
        }
        total
    }
}

#[derive(Debug)]
struct Account<'a> {
    name: &'a str,
    acc_type: AccountType,
    created_date: DateTime<Utc>,
    transactions: Vec<Transaction>,
}

impl<'a> Account<'a> {
    fn new(name: &'a str, acc_type: AccountType) -> Self {
        Self {
            name,
            acc_type,
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
    amount: f32,
    execution_date: DateTime<Utc>,
    trx_type: TransactionType,
}

impl Transaction {
    fn new(amount: f32, trx_type: TransactionType) -> Self {
        Transaction {
            amount,
            trx_type,
            execution_date: Utc::now(),
        }
    }
}

fn main() {
    let mut wallet = Wallet {
        accounts: vec![
            Account::new("test-asset", AccountType::Asset),
            Account::new("test-liability", AccountType::Liability),
        ],
    };

    for acc in wallet.accounts.iter_mut() {
        acc.transactions
            .push(Transaction::new(1000.0, TransactionType::Debit));
        acc.transactions
            .push(Transaction::new(1000.0, TransactionType::Debit));
        acc.transactions
            .push(Transaction::new(500.0, TransactionType::Credit));
        println!("{}: {}", acc.name, acc.balance());
    }
    println!("Wallet: {}", wallet.net_worth());
}

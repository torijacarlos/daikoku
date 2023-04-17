use super::Account;

#[derive(Debug)]
pub struct Wallet<'a> {
    pub id: Option<u32>,
    pub username: &'a str,
    pub accounts: Vec<Account<'a>>,
}

impl<'a> Wallet<'a> {
    pub fn new(username: &'a str) -> Self {
        Self {
            id: None,
            username,
            accounts: vec![],
        }
    }

    pub fn net_worth(&self) -> f32 {
        let mut total = 0.0;
        for acc in &self.accounts {
            total += acc.balance();
        }
        total
    }
}



#[derive(Debug)]
pub enum AccountType {
    Asset,
    Liability,
    Expense,
    Income,
    Equity,
}

impl TryFrom<String> for AccountType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let result = match &value.to_lowercase()[..] {
            "asset" => Self::Asset,
            "liability" => Self::Liability,
            "expense" => Self::Expense,
            "income" => Self::Income,
            "equity" => Self::Equity,
            _ => return Err(format!("Unhandled Account type: {}", value))
        };
        Ok(result)
    }
}

#[derive(Debug)]
pub enum TransactionType {
    Debit,
    Credit,
}

impl TryFrom<String> for TransactionType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let result = match &value.to_lowercase()[..] {
            "debit" => Self::Debit,
            "credit" => Self::Credit,
            _ => return Err(format!("Unhandled Transaction type: {}", value))
        };
        Ok(result)
    }
}


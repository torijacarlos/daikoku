use sqlx::{database::HasValueRef, Database, Decode};

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum AccountType {
    Asset,
    Liability,
    Expense,
    Income,
    Equity,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match &self {
            Self::Asset => "Asset",
            Self::Liability => "Liability",
            Self::Expense => "Expense",
            Self::Income => "Income",
            Self::Equity => "equity",
        }
    }
}

impl<'r, D: Database> Decode<'r, D> for AccountType
where
    // we want to delegate some of the work to string decoding so let's make sure strings
    // are supported by the database
    &'r str: Decode<'r, D>,
{
    fn decode(
        value: <D as HasValueRef<'r>>::ValueRef,
    ) -> Result<AccountType, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <&str as Decode<D>>::decode(value)?.to_string();
        Ok(TryInto::<AccountType>::try_into(value)?)
    }
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
            _ => return Err(format!("Unhandled Account type: {}", value)),
        };
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionType {
    Debit,
    Credit,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match &self {
            Self::Debit => "Debit",
            Self::Credit => "Credit",
        }
    }
}

impl TryFrom<String> for TransactionType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let result = match &value.to_lowercase()[..] {
            "debit" => Self::Debit,
            "credit" => Self::Credit,
            _ => return Err(format!("Unhandled Transaction type: {}", value)),
        };
        Ok(result)
    }
}

impl<'r, D: Database> Decode<'r, D> for TransactionType
where
    // we want to delegate some of the work to string decoding so let's make sure strings
    // are supported by the database
    &'r str: Decode<'r, D>,
{
    fn decode(
        value: <D as HasValueRef<'r>>::ValueRef,
    ) -> Result<TransactionType, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <&str as Decode<D>>::decode(value)?.to_string();
        Ok(TryInto::<TransactionType>::try_into(value)?)
    }
}

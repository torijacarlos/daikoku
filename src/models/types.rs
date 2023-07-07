use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub enum AccountType {
    #[default]
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

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum TransactionType {
    #[default]
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

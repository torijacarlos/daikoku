use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::TransactionType;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub amount: BigDecimal,
    pub execution_date: DateTime<Utc>,
    pub trx_type: TransactionType,
}

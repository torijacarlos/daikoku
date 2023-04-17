use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TransactionType;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Option<Uuid>,
    pub amount: BigDecimal,
    pub execution_date: DateTime<Utc>,
    pub trx_type: TransactionType,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
}

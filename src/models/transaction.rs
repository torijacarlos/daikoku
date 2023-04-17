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

impl Transaction {
    pub fn new() -> Self {
        Self {
            id: Some(Uuid::new_v4()),
            execution_date: Utc::now(),
            created_date: Utc::now(),
            updated_date: Utc::now(),
            ..Default::default()
        }
    }
}

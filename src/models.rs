use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransferEvent {
    pub id: String,
    pub transaction_hash: String,
    pub log_index: i64,
    pub block_number: i64,
    pub block_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub value: String,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DatabaseStats {
    pub total_transfers: i64,
    pub unique_addresses: i64,
    pub latest_block: Option<i64>,
    pub earliest_block: Option<i64>,
}


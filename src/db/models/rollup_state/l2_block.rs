use crate::db::TimestampDbType;
use serde::Serialize;

#[derive(sqlx::FromRow, Serialize, Debug, Clone)]
pub struct L2Block {
    pub block_id: i64, // TODO: keep this consistent with the smart contract
    pub new_root: String,
    // block_status: unsubmitted, submitted, confirmed
    pub detail: serde_json::Value,
    pub created_time: TimestampDbType,
}

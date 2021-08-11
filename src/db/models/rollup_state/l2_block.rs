use crate::db::TimestampDbType;
use serde::Serialize;

#[derive(sqlx::Type, Serialize, Debug, Clone)]
#[sqlx(type_name = "block_status", rename_all = "snake_case")]
pub enum BlockStatus {
    Uncommited,
    Commited,
    Verified,
}

#[derive(sqlx::FromRow, Serialize, Debug, Clone)]
pub struct L2Block {
    pub block_id: i64, // TODO: keep this consistent with the smart contract
    pub new_root: String,
    pub status: BlockStatus,
    // TODO: tx_hash
    pub detail: serde_json::Value,
    pub created_time: TimestampDbType,
}

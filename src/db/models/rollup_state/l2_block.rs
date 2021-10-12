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
    pub l1_tx_hash: Option<String>, // use String for now, consider switch to <u8> in the future.
    pub detail: serde_json::Value,
    pub created_time: TimestampDbType,
}

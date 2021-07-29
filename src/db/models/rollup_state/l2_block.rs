use crate::db::TimestampDbType;
use serde::Serialize;

#[derive(sqlx::FromRow, Serialize, Debug, Clone)]
pub struct L2Block {
    pub block_id: i64,
    pub new_root: String,
    pub witness: serde_json::Value,
    pub created_time: TimestampDbType,
}

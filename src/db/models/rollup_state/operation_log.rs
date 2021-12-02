use crate::db::TimestampDbType;
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub id: i64,
    pub time: TimestampDbType,
    pub method: String,
    pub params: String, // TODO: change it to jsonb
}

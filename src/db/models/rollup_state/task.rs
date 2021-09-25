use crate::db::TimestampDbType;
use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Debug, Clone)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
pub enum TaskStatus {
    Inited,
    Proving,
    Proved,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum CircuitType {
    BLOCK,
}

#[derive(sqlx::FromRow, Serialize, Debug, Clone)]
pub struct Task {
    pub task_id: String,
    pub circuit: CircuitType,
    pub block_id: i64,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub public_input: Option<Vec<u8>>,
    pub proof: Option<Vec<u8>>,
    pub status: TaskStatus,
    pub prover_id: Option<String>,
    pub created_time: TimestampDbType,
    pub updated_time: TimestampDbType,
}

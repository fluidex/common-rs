use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct AccountDesc {
    pub id: i32, // TODO: i32 or i64?
    pub l1_address: String,
    pub l2_pubkey: String,
}

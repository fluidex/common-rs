pub mod tablenames {
    pub const ACCOUNT: &str = "account";
    pub const L2_BLOCK: &str = "l2block";
    pub const OPERATION_LOG: &str = "operation_log";
    pub const TASK: &str = "task";
}

pub mod account;
pub mod l2_block;
pub mod operation_log;
pub mod task;

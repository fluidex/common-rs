pub mod tablenames {
    pub const L2_BLOCK: &str = "l2block";
    pub const TASK: &str = "task";
}

mod l2_block;
pub use l2_block::*;

mod task;
pub use task::*;

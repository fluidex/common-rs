use once_cell::sync::Lazy;

pub mod serde;
pub mod types;

pub use types::Fr;

/// re-exports common dependencies
pub use babyjubjub_rs;
pub use ff;
pub use fnv;
pub use num_bigint;
pub use num_traits;
pub use poseidon_rs;
pub use rust_decimal;

/// [`poseidon_rs::Poseidon`] global
pub static POSEIDON_HASHER: Lazy<poseidon_rs::Poseidon> = Lazy::new(|| poseidon_rs::Poseidon::new());
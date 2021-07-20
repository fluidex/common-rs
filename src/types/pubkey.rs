use std::convert::TryInto;

use crate::babyjubjub_rs::decompress_point;
pub use crate::babyjubjub_rs::Point as Pubkey;

#[derive(Debug, thiserror::Error)]
pub enum PubkeyExtError {
    #[error(transparent)]
    HexDecode(#[from] hex::FromHexError),
    #[error("invalid pubkey packed length {} instead of 32", .0.len())]
    InvalidLength(Vec<u8>),
    #[error("{0}")]
    InvalidPoint(String),
}

type Result<T, E = PubkeyExtError> = std::result::Result<T, E>;

/// Pubkey extension
pub trait PubkeyExt: Sized {
    /// Parse a packed pubkey hex string
    fn from_str(pubkey: &str) -> Result<Self>;
}

impl PubkeyExt for Pubkey {
    fn from_str(pubkey: &str) -> Result<Self> {
        use PubkeyExtError::*;

        let pubkey = pubkey.trim_start_matches("0x");
        let pubkey_packed = hex::decode(pubkey)?;
        decompress_point(pubkey_packed.try_into().map_err(InvalidLength)?).map_err(InvalidPoint)
    }
}

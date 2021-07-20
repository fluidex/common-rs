use std::convert::TryInto;

use crate::babyjubjub_rs::decompress_signature;
pub use crate::babyjubjub_rs::Signature;

#[derive(Debug, thiserror::Error)]
pub enum SignatureExtError {
    #[error(transparent)]
    HexDecode(#[from] hex::FromHexError),
    #[error("invalid signature packed length {} instead of 64", .0.len())]
    InvalidLength(Vec<u8>),
    #[error("{0}")]
    InvalidPoint(String),
}

type Result<T, E = SignatureExtError> = std::result::Result<T, E>;

/// [`Signature`] extension
pub trait SignatureExt: Sized {
    /// Parse a packed signature hex string
    fn from_str(pubkey: &str) -> Result<Self>;
}

impl SignatureExt for Signature {
    fn from_str(signature: &str) -> Result<Signature> {
        use SignatureExtError::*;

        let signature = signature.trim_start_matches("0x");
        let sig_packed_vec = hex::decode(signature)?;
        decompress_signature(&sig_packed_vec.try_into().map_err(InvalidLength)?)
            .map_err(InvalidPoint)
    }
}

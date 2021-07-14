//! Common types definitions
use ff::*;
use std::str::FromStr;

pub use fnv::FnvHashMap as MerkleValueMapType;

/// re-exports [`num_bigint::BigInt`]
pub use num_bigint::BigInt;
/// re-exports [`rust_decimal::Decimal`]
pub use rust_decimal::Decimal;

pub type Fr = poseidon_rs::Fr;

#[derive(Debug, thiserror::Error)]
pub enum FrExtError {
    #[error("invalid value for bool")]
    InvalidBool,
    #[error("invalid slice length for Fr")]
    InvalidLength,
    #[error(transparent)]
    BufferError(#[from] std::io::Error),
    #[error(transparent)]
    PrimeFieldDecodingError(#[from] ff::PrimeFieldDecodingError),
}

type Result<T, E = FrExtError> = std::result::Result<T, E>;

pub trait FrExt: Sized {
    fn shl(&self, x: u32) -> Self;
    fn sub(&self, b: &Fr) -> Self;
    fn add(&self, b: &Fr) -> Self;
    fn from_u32(x: u32) -> Self;
    fn from_u64(x: u64) -> Self;
    fn from_bigint(x: BigInt) -> Self;
    fn from_str(x: &str) -> Self;
    fn from_slice(slice: &[u8]) -> Result<Self>;
    fn to_u32(&self) -> u32;
    fn to_i64(&self) -> i64;
    fn to_bigint(&self) -> BigInt;
    fn to_decimal_string(&self) -> String;
    fn to_decimal(&self, scale: u32) -> Decimal;
    fn to_vec_be(&self) -> Vec<u8>;
    fn to_bool(&self) -> Result<bool>;
}

impl FrExt for Fr {
    fn shl(&self, x: u32) -> Self {
        let mut repr = self.into_repr();
        repr.shl(x);
        Fr::from_repr(repr).unwrap()
    }

    fn sub(&self, b: &Fr) -> Self {
        let mut r = *self;
        r.sub_assign(b);
        r
    }

    fn add(&self, b: &Fr) -> Self {
        let mut r = *self;
        r.add_assign(b);
        r
    }

    fn from_u32(x: u32) -> Self {
        PrimeField::from_str(&format!("{}", x)).unwrap()
    }

    fn from_u64(x: u64) -> Self {
        Fr::from_repr(poseidon_rs::FrRepr::from(x)).unwrap()
    }

    fn from_bigint(x: BigInt) -> Self {
        let mut s = x.to_str_radix(16);
        if s.len() % 2 != 0 {
            // convert "f" to "0f"
            s.insert(0, '0');
        }
        from_hex(&s).unwrap()
    }

    fn from_str(x: &str) -> Self {
        if x.starts_with("0x") {
            Self::from_slice(&hex::decode(x.trim_start_matches("0x")).unwrap()).unwrap()
        } else {
            let i = BigInt::from_str(x).unwrap();
            Self::from_bigint(i)
        }
    }

    fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() > 32 {
            return Err(FrExtError::InvalidLength);
        }
        let mut repr = <Fr as PrimeField>::Repr::default();

        // prepad 0
        let mut buf = slice.to_vec();
        let required_length = repr.as_ref().len() * 8;
        buf.reverse();
        buf.resize(required_length, 0);
        buf.reverse();

        repr.read_be(&buf[..])?;
        Ok(Fr::from_repr(repr)?)
    }

    fn to_u32(&self) -> u32 {
        Self::to_decimal_string(self).parse::<u32>().unwrap()
    }

    fn to_i64(&self) -> i64 {
        Self::to_decimal_string(self).parse::<i64>().unwrap()
    }

    fn to_bigint(&self) -> BigInt {
        BigInt::parse_bytes(to_hex(self).as_bytes(), 16).unwrap()
    }

    fn to_decimal_string(&self) -> String {
        Self::to_bigint(self).to_str_radix(10)
    }

    fn to_decimal(&self, scale: u32) -> Decimal {
        Decimal::new(Self::to_i64(self), scale)
    }

    fn to_vec_be(&self) -> Vec<u8> {
        let repr = self.into_repr();
        let required_length = repr.as_ref().len() * 8;
        let mut buf: Vec<u8> = Vec::with_capacity(required_length);
        repr.write_be(&mut buf).unwrap();
        buf
    }

    fn to_bool(&self) -> Result<bool> {
        if self.is_zero() {
            Ok(false)
        } else if self == &Fr::one() {
            Ok(true)
        } else {
            Err(FrExtError::InvalidBool)
        }
    }
}

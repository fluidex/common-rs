//! Extra support for misc types in serde.
//!
//! Example:
//! ```
//! use serde::{Serialize, Deserialize};
//! use fluidex_common::serde::*;
//! use fluidex_common::Fr;
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Message {
//!     #[serde(with = "HexArray")]
//!     signature: [u8; 64],
//!     #[serde(with = "FrBytes")]
//!     from: Fr,
//!     #[serde(with = "FrStr")]
//!     to: Fr,
//! }
//! ```
use core::convert::TryInto;
use core::fmt;
use core::marker::PhantomData;
use core::str::FromStr;

use num_bigint::BigInt;
use serde::de::{Deserializer, Error, Unexpected, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

use crate::types::{Fr, FrExt, MerkleValueMapType};
use std::hash::Hash;

/// Helper trait add serde support to `[u8; N]` using hex encoding.
pub trait HexArray<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

/// Helper trait add serde support to `Fr` using bytes encoding.
pub trait FrBytes<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

/// Helper trait add serde support to `Fr` using big decimal string literal encoding.
pub trait FrStr<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl<'de, const N: usize> HexArray<'de> for [u8; N] {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(hex::encode(&self).as_str())
    }

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HexArrayVisitor<T> {
            value: PhantomData<T>,
        }

        impl<'de, const N: usize> Visitor<'de> for HexArrayVisitor<[u8; N]> {
            type Value = [u8; N];

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an hex encoded array of length {}", N)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                hex::decode(v)
                    .ok()
                    .and_then(|v| v.try_into().ok())
                    .ok_or_else(|| Error::invalid_type(Unexpected::Str(v), &self))
            }
        }

        let visitor = HexArrayVisitor { value: PhantomData };
        deserializer.deserialize_str(visitor)
    }
}

impl<'de> FrBytes<'de> for Fr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.to_vec_be())
    }

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FrBytesVisitor;

        impl<'de> Visitor<'de> for FrBytesVisitor {
            type Value = Fr;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Fr in be bytes repr")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if let Ok(fr) = Fr::from_slice(v) {
                    Ok(fr)
                } else {
                    Err(Error::invalid_type(Unexpected::Bytes(v), &self))
                }
            }
        }

        deserializer.deserialize_bytes(FrBytesVisitor)
    }
}

impl<'de, K> FrBytes<'de> for MerkleValueMapType<K, Fr>
where
    K: Eq + Hash + Serialize + Deserialize<'de>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wrapper(#[serde(with = "FrBytes")] Fr);

        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            map.serialize_entry(k, &Wrapper(*v))?;
        }
        map.end()
    }

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wrapper(#[serde(with = "FrBytes")] Fr);

        let map = MerkleValueMapType::<K, Wrapper>::deserialize(deserializer)?;

        Ok(map.into_iter().map(|(k, Wrapper(v))| (k, v)).collect())
    }
}

impl<'de> FrStr<'de> for Fr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_decimal_string().as_str())
    }

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FrStrVisitor;

        impl<'de> Visitor<'de> for FrStrVisitor {
            type Value = Fr;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Fr in decimal str repr")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if let Ok(fr) = BigInt::from_str(v) {
                    Ok(Fr::from_bigint(fr))
                } else {
                    Err(Error::invalid_type(Unexpected::Str(v), &self))
                }
            }
        }

        deserializer.deserialize_str(FrStrVisitor)
    }
}

impl<'de, K> FrStr<'de> for MerkleValueMapType<K, Fr>
where
    K: Eq + Hash + Serialize + Deserialize<'de>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wrapper(#[serde(with = "FrStr")] Fr);

        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            map.serialize_entry(k, &Wrapper(*v))?;
        }
        map.end()
    }

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wrapper(#[serde(with = "FrStr")] Fr);

        let map = MerkleValueMapType::<K, Wrapper>::deserialize(deserializer)?;

        Ok(map.into_iter().map(|(k, Wrapper(v))| (k, v)).collect())
    }
}

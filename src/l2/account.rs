use crate::ff::Field;
use crate::types::FrExt;
use crate::Fr;
use anyhow::Result;
use babyjubjub_rs::{self, Point, PrivateKey};
use ethers::core::k256::ecdsa::digest::generic_array::GenericArray;
use ethers::core::k256::ecdsa::digest::{BlockInput, Digest, FixedOutput, Output, Reset, Update};
use ethers::core::k256::ecdsa::recoverable::Signature as RecoverableSignature;
use ethers::core::k256::ecdsa::signature::DigestSigner;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::core::k256::elliptic_curve::{consts::U64, FieldBytes};
use ethers::core::k256::Secp256k1;
use ethers::core::types::{H256, U256};
use ethers::prelude::Signature as EthersSignature;
use ethers::signers::to_eip155_v;
use ethers::utils::hash_message;
use once_cell::sync::Lazy;
use std::fmt;

pub type SignatureBJJ = babyjubjub_rs::Signature;

static CHAIN_ID: Lazy<u32> = Lazy::new(|| {
    std::env::var("CHAIN_ID")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u32>()
        .unwrap_or(1)
});
static CREATE_L2_ACCOUNT_MSG: Lazy<String> =
    Lazy::new(|| format!("FLUIDEX_L2_ACCOUNT\nChain ID: {}.", *CHAIN_ID));

pub struct L2Account {
    priv_key: PrivateKey,
    pub pub_key: Point,
    pub ax: Fr,
    pub ay: Fr,
    pub sign: Fr,
    pub bjj_pub_key: String,
}

impl fmt::Debug for L2Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("L2Account")
            .field("priv_key", &self.priv_key.scalar_key())
            .field("pub_key", &self.pub_key)
            .field("bjj_pub_key", &self.bjj_pub_key)
            .finish()
    }
}

impl L2Account {
    pub fn new(private_key: &str) -> Result<Self, String> {
        let private_key_bytes = hex::decode(private_key.trim_start_matches("0x")).unwrap();
        let private_key = SigningKey::from_bytes(&private_key_bytes).unwrap();
        let signature = sign_msg_with_signing_key(&private_key, &*CREATE_L2_ACCOUNT_MSG);
        let seed = &signature.to_vec()[0..32];

        let priv_key = PrivateKey::import(seed.to_vec())?;
        let pub_key: Point = priv_key.public();
        let ax = pub_key.x;
        let ay = pub_key.y;
        let bjj_compressed = pub_key.compress();
        let sign = if bjj_compressed[31] & 0x80 != 0x00 {
            Fr::one()
        } else {
            Fr::zero()
        };
        let bjj_pub_key = hex::encode(bjj_compressed);

        Ok(Self {
            priv_key,
            pub_key,
            ax,
            ay,
            sign,
            bjj_pub_key,
        })
    }

    pub fn sign_hash_raw(&self, hash: Fr) -> Result<SignatureBJJ, String> {
        self.priv_key.sign(hash.to_bigint())
    }
    pub fn sign_hash(&self, hash: Fr) -> Result<Signature, String> {
        let sig = self.sign_hash_raw(hash)?;
        let s = Fr::from_bigint(sig.s);
        Ok(Signature {
            hash,
            s,
            r8x: sig.r_b8.x,
            r8y: sig.r_b8.y,
        })
    }
    pub fn sign_hash_packed(&self, hash: Fr) -> Result<[u8; 64], String> {
        Ok(self.priv_key.sign(hash.to_bigint())?.compress())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Signature {
    pub hash: Fr,
    pub s: Fr,
    pub r8x: Fr,
    pub r8y: Fr,
}

impl Signature {
    pub fn from_raw(hash: Fr, sig: &SignatureBJJ) -> Self {
        Self {
            hash,
            s: Fr::from_bigint(sig.clone().s),
            r8x: sig.clone().r_b8.x,
            r8y: sig.clone().r_b8.y,
        }
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self {
            hash: Fr::zero(),
            s: Fr::zero(),
            r8x: Fr::zero(),
            r8y: Fr::zero(),
        }
    }
}

/// Signs the message with the signing key and returns the ethers core signature.
/// Copied from https://github.com/gakonst/ethers-rs/blob/01cc80769c291fc80f5b1e9173b7b580ae6b6413/ethers-signers/src/wallet/mod.rs#L71
fn sign_msg_with_signing_key(priv_key: &SigningKey, msg: &str) -> EthersSignature {
    let msg_hash = hash_message(msg);
    let digest = Sha256Proxy::from(msg_hash);
    let recoverable_sig: RecoverableSignature = priv_key.sign_digest(digest);

    //TODO: what if we want to use different CHAIN_ID for layer 1, but keep using 1 for layer 2?
    let v = to_eip155_v(recoverable_sig.recovery_id(), *CHAIN_ID as u64);

    let r_bytes: FieldBytes<Secp256k1> = recoverable_sig.r().into();
    let s_bytes: FieldBytes<Secp256k1> = recoverable_sig.s().into();
    let r = U256::from_big_endian(r_bytes.as_slice());
    let s = U256::from_big_endian(s_bytes.as_slice());

    EthersSignature { r, s, v }
}

// Helper type for calling sign_digest method of SigningKey.
// Copied from https://github.com/gakonst/ethers-rs/blob/01cc80769c291fc80f5b1e9173b7b580ae6b6413/ethers-signers/src/wallet/hash.rs#L11
type Sha256Proxy = ProxyDigest<sha2::Sha256>;

#[derive(Clone)]
enum ProxyDigest<D: Digest> {
    Proxy(Output<D>),
    Digest(D),
}

impl<D: Digest + Clone> From<H256> for ProxyDigest<D>
where
    GenericArray<u8, <D as Digest>::OutputSize>: Copy,
{
    fn from(src: H256) -> Self {
        ProxyDigest::Proxy(*GenericArray::from_slice(src.as_bytes()))
    }
}

impl<D: Digest> Default for ProxyDigest<D> {
    fn default() -> Self {
        ProxyDigest::Digest(D::new())
    }
}

impl<D: Digest> Update for ProxyDigest<D> {
    // we update only if we are digest
    fn update(&mut self, data: impl AsRef<[u8]>) {
        match self {
            ProxyDigest::Digest(ref mut d) => {
                d.update(data);
            }
            ProxyDigest::Proxy(..) => {
                unreachable!("can not update if we are proxy");
            }
        }
    }

    // we chain only if we are digest
    fn chain(self, data: impl AsRef<[u8]>) -> Self {
        match self {
            ProxyDigest::Digest(d) => ProxyDigest::Digest(d.chain(data)),
            ProxyDigest::Proxy(..) => {
                unreachable!("can not update if we are proxy");
            }
        }
    }
}

impl<D: Digest> Reset for ProxyDigest<D> {
    // make new one
    fn reset(&mut self) {
        *self = Self::default();
    }
}

// Use Sha256 with 512 bit blocks
impl<D: Digest> BlockInput for ProxyDigest<D> {
    type BlockSize = U64;
}

impl<D: Digest> FixedOutput for ProxyDigest<D> {
    // we default to the output of the original digest
    type OutputSize = D::OutputSize;

    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        match self {
            ProxyDigest::Digest(d) => {
                *out = d.finalize();
            }
            ProxyDigest::Proxy(p) => {
                *out = p;
            }
        }
    }

    fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        let s = std::mem::take(self);
        s.finalize_into(out);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        const KEY: &str = "0x78fcdf46b07b7a0033f97646dff6528b116c8230b66bb82f86c0fb572349cf9c";
        const PUBKEY: &str = "0xe74a2954787f9fab013d7186388f7a610326bebf";
        const BJJ_PUBKEY: &str = "80c117e2be91e44526e14fa786c565d15b79585d1ae1ef229cfcdf47570f241c";
        let account = L2Account::new(KEY).unwrap();
        assert_eq!(account.bjj_pub_key, BJJ_PUBKEY);
    }
}

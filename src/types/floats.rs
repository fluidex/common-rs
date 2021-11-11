use super::{BigInt, Decimal};
use crate::num_traits::{identities::Zero, int::PrimInt, FromPrimitive, Pow, Signed, ToPrimitive};
use crate::types::{Fr, FrExt};

/// a POSTIVE float representation with 1 byte exponent and NBITS significand, the bits for exponent is 8 - NBITS % 8
//  so total bits for encoding a number would be always aligned to the byte edge
#[derive(Debug, Clone, Copy)]
pub struct Floats<T: PrimInt, const NBITS: usize> {
    pub exponent: u8,
    //represent a unsigned int in little-endian fashion (last element for the most signifcant byte)
    pub significand: T,
}

#[derive(Debug, thiserror::Error)]
pub enum FloatsError {
    #[error("decimal precision error {0} {1}")]
    Precision(Decimal, u32),
    #[error("invalid precision {0} {1} {2}")]
    InvalidPrecision(Decimal, u32, Decimal),
    #[error(transparent)]
    TryFromSlice(#[from] std::array::TryFromSliceError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Demical(rust_decimal::Error),
    #[error("exponent is too big")]
    ExponentTooBig,
    #[error("number {0} too big to be saved")]
    NumberTooBig(BigInt),
}

type Result<T, E = FloatsError> = std::result::Result<T, E>;

impl<T: PrimInt + Zero, const NBITS: usize> Floats<T, NBITS> {
    fn sig_to_bigint(self) -> BigInt {
        //cast to the largest int (128bit) possible
        if T::min_value() < T::zero() {
            BigInt::from(self.significand.to_i128().unwrap())
        } else {
            BigInt::from(self.significand.to_u128().unwrap())
        }
    }

    pub fn encode_len() -> usize {
        (NBITS + 8 - NBITS % 8) / 8
    }

    pub fn zero() -> Self {
        Self {
            significand: T::zero(),
            exponent: 0,
        }
    }

    pub fn to_bigint(self) -> BigInt {
        //cast to the largest int (128bit) possible
        BigInt::from(10).pow(self.exponent) * self.sig_to_bigint()
    }

    pub fn to_encoded_int(self) -> Result<BigInt> {
        //cast to the largest int (128bit) possible
        let max_exp = (1 << (8 - NBITS % 8)) - 1;
        if self.exponent > max_exp as u8 {
            return Err(FloatsError::ExponentTooBig);
        }

        if self.significand == T::zero() {
            return Ok(BigInt::zero());
        }

        let sig = self.sig_to_bigint();
        if sig.is_positive() {
            Ok((BigInt::from(self.exponent) << NBITS) + sig)
        } else {
            Ok((BigInt::from(self.exponent) << NBITS) + ((BigInt::from(1) << NBITS) + sig))
        }
    }

    pub fn to_fr(self) -> Fr {
        Fr::from_bigint(self.to_bigint())
    }

    //encode to big-endian bytes, with the exponent parts at the beginning
    //suppose it could be accommodate to an u128 integer
    pub fn encode(self) -> Vec<u8> {
        let bi = self.to_encoded_int().unwrap();
        let (_, mut bytes) = bi.to_bytes_be();

        let mut head_zeros = vec![0u8; Self::encode_len() - bytes.len()];
        head_zeros.append(&mut bytes);
        head_zeros
    }

    pub fn from_encoded_bigint(bi: BigInt) -> Result<Self> {
        assert!(NBITS < 120);

        //we do not need the signed for encoded integer
        let bi = if bi.is_positive() { bi } else { -bi };

        let signi_mask: BigInt = (BigInt::from(1) << NBITS) - 1;
        let significand = &bi & &signi_mask;
        let exponent = &bi >> NBITS;
        let significand = if T::min_value() < T::zero() {
            let signed_max = BigInt::from(1) << (NBITS - 1);
            if significand <= signed_max {
                T::from(significand.to_i128().unwrap()).ok_or(FloatsError::NumberTooBig(bi))?
            } else {
                let significand = -(signi_mask - significand + BigInt::from(1));
                T::from(significand.to_i128().unwrap()).ok_or(FloatsError::NumberTooBig(bi))?
            }
        } else {
            T::from(significand.to_u128().unwrap()).ok_or(FloatsError::NumberTooBig(bi))?
        };

        let exponent = exponent.to_u8().ok_or(FloatsError::ExponentTooBig)?;

        Ok(Self {
            exponent,
            significand,
        })
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        Self::from_encoded_bigint(BigInt::from_bytes_be(num_bigint::Sign::Plus, data))
    }

    pub fn to_decimal(self, prec: u32) -> Decimal {
        // for example, (significand:1, exponent:17) means 10**17, when prec is 18,
        // it is 0.1 (ETH)
        //TODO: demical can not handle full 128bit integer so we need to verify that,
        //in case the significand can be hold by Decimal it must be able to be converted
        //into i128 safetily
        let mut ret = Decimal::from_i128(self.significand.to_i128().unwrap()).unwrap();
        if (self.exponent as u32) < prec {
            ret.set_scale(prec - self.exponent as u32).unwrap();
            ret
        } else {
            ret * Decimal::new(10, 0).pow(self.exponent as u64 - prec as u64)
        }
    }

    pub fn from_bigint(bi: BigInt) -> Result<Self> {
        let eff_bits = T::zero().count_zeros() as usize;
        assert!(eff_bits > NBITS && eff_bits <= 128);

        //TODO: we are not able to handle T as u128 yet
        let test_low_bound = T::min_value() >> (eff_bits - NBITS);
        let test_high_bound = T::max_value() >> (eff_bits - NBITS);

        let max_exp = (1 << (8 - NBITS % 8)) - 1;

        let mut exponent = 0u8;
        let mut encode_int = bi.clone();
        let mut test_sig: BigInt = bi.clone() / 10;
        while encode_int == test_sig.clone() * 10 && exponent <= max_exp as u8 {
            encode_int = test_sig;
            exponent += 1;
            test_sig = encode_int.clone() / 10;
        }

        println!("encode_int {}", encode_int);

        let significand = if T::min_value() < T::zero() {
            T::from(
                encode_int
                    .to_i128()
                    .ok_or_else(|| FloatsError::NumberTooBig(bi.clone()))?,
            )
            .ok_or_else(|| FloatsError::NumberTooBig(bi.clone()))?
        } else {
            T::from(
                encode_int
                    .to_u128()
                    .ok_or_else(|| FloatsError::NumberTooBig(bi.clone()))?,
            )
            .ok_or_else(|| FloatsError::NumberTooBig(bi.clone()))?
        };

        if significand > test_high_bound || significand < test_low_bound {
            Err(FloatsError::NumberTooBig(bi))
        } else {
            Ok(Self {
                exponent,
                significand,
            })
        }
    }

    //update from Decimal and round
    pub fn from_decimal(d: &Decimal, prec: u32) -> Result<Self> {
        let eff_bits = T::zero().count_zeros() as usize;
        assert!(eff_bits > NBITS && eff_bits <= 128);

        //TODO: we are not able to handle T as u128 yet
        let test_low_bound = (T::min_value() >> (eff_bits - NBITS)).to_i128().unwrap();
        let test_high_bound = (T::max_value() >> (eff_bits - NBITS)).to_i128().unwrap();

        if d.is_zero() {
            return Ok(Self {
                exponent: 0,
                significand: T::zero(),
            });
        }
        // if d is "0.1" and prec is 18, result is (significand:1, exponent:17)
        let mut n = *d;
        let mut exponent = if n.scale() < prec {
            prec - d.scale()
        } else {
            0u32
        };

        n.set_scale(if n.scale() < prec {
            0
        } else {
            n.scale() - prec
        })
        .map_err(FloatsError::Demical)?;
        if n != n.floor() {
            return Err(FloatsError::Precision(n, prec));
        }
        let mut test = n.to_i128().unwrap();
        loop {
            let next = test / 10i128;
            if test > test_high_bound || test < test_low_bound || next * 10i128 == test {
                exponent += 1;
                test = next;
            } else {
                break;
            }
        }

        Ok(Self {
            exponent: exponent as u8,
            significand: T::from(test).unwrap(),
        })
    }
}

pub type Float40 = Floats<i64, 35>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    //type Float864 = Floats<i64, 4>;
    #[test]
    fn test_primitive() {
        let m1 = Float40 {
            exponent: 0,
            significand: 65536,
        };
        assert_eq!(m1.to_bigint(), BigInt::from(65536));
        assert_eq!(m1.to_encoded_int().unwrap(), BigInt::from(65536));
        assert_eq!(
            BigInt::from(65536),
            Float40::from_encoded_bigint(BigInt::from(65536))
                .unwrap()
                .to_bigint()
        );
        let m2 = Float40 {
            exponent: 1,
            significand: 16777216,
        };
        assert_eq!(m2.to_bigint(), BigInt::from(167772160));
        assert_eq!(m2.to_encoded_int().unwrap(), BigInt::from(34376515584u128));
        assert_eq!(
            BigInt::from(167772160),
            Float40::from_encoded_bigint(BigInt::from(34376515584u128))
                .unwrap()
                .to_bigint()
        );
        let m3 = Floats::<i32, 24> {
            exponent: 1,
            significand: 65536,
        };
        assert_eq!(m3.to_bigint(), BigInt::from(655360));
        assert_eq!(m3.to_encoded_int().unwrap(), BigInt::from(16842752));
        assert_eq!(
            BigInt::from(655360),
            Floats::<i32, 24>::from_encoded_bigint(BigInt::from(16842752))
                .unwrap()
                .to_bigint()
        );
        let m4 = Floats::<i32, 16> {
            exponent: 0,
            significand: -32767,
        };
        assert_eq!(m4.to_bigint(), BigInt::from(-32767));
        assert_eq!(m4.to_encoded_int().unwrap(), BigInt::from(32769));
        assert_eq!(
            BigInt::from(-32767),
            Floats::<i32, 16>::from_encoded_bigint(BigInt::from(32769))
                .unwrap()
                .to_bigint()
        );
        let m5 = Floats::<i32, 18> {
            exponent: 2,
            significand: -32767,
        };
        assert_eq!(m5.to_bigint(), BigInt::from(-3276700));
        assert_eq!(m5.to_encoded_int().unwrap(), BigInt::from(753665));
        assert_eq!(
            BigInt::from(-3276700),
            Floats::<i32, 18>::from_encoded_bigint(BigInt::from(753665))
                .unwrap()
                .to_bigint()
        );
        let m6 = Float40 {
            exponent: 1,
            significand: -1,
        };
        assert_eq!(m6.to_bigint(), BigInt::from(-10));
        assert_eq!(m6.to_encoded_int().unwrap(), BigInt::from(68719476735u128));
        assert_eq!(
            BigInt::from(-10),
            Float40::from_encoded_bigint(BigInt::from(68719476735u128))
                .unwrap()
                .to_bigint()
        );
    }

    #[test]
    fn test_primitive2() {
        let m1 = Float40::from_bigint(BigInt::from(1000)).unwrap();
        assert_eq!(m1.exponent, 3);
        assert_eq!(m1.significand, 1);

        let m2 = Float40::from_bigint(BigInt::from(100000000000u128)).unwrap();
        assert_eq!(m2.exponent, 11);
        assert_eq!(m2.significand, 1);

        let m3 = Float40::from_bigint(BigInt::from(999990)).unwrap();
        assert_eq!(m3.exponent, 1);
        assert_eq!(m3.significand, 99999);

        let m3 = Float40::from_bigint(BigInt::from(-777770)).unwrap();
        assert_eq!(m3.exponent, 1);
        assert_eq!(m3.significand, -77777);

        //notice the bound is -17179869183 ~ 17179869183
        let m4 = Float40::from_bigint(BigInt::from(159999999990u128)).unwrap();
        assert_eq!(m4.exponent, 1);
        assert_eq!(m4.significand, 15999999999i64);

        Float40::from_bigint(BigInt::from(159999999999u128)).expect_err("expect too big error");

        //extremely big but can be encoded
        let m5 = Float40::from_bigint(
            BigInt::from_str("-18330000000000000000000000000000000000").unwrap(),
        )
        .unwrap();
        assert_eq!(m5.exponent, 32);
        assert_eq!(m5.significand, -183300);
    }

    #[test]
    fn test_encode() {
        let m1 = Float40 {
            exponent: 0,
            significand: 65536,
        };
        let ret = m1.encode();
        assert_eq!(ret.len(), 5);
        assert_eq!(ret[0], 0);
        assert_eq!(ret[2], 1);
        assert_eq!(ret[4], 0);
        let m2 = Float40 {
            exponent: 1,
            significand: 16777216,
        };
        let ret = m2.encode();
        assert_eq!(ret.len(), 5);
        assert_eq!(ret[0], 8);
        assert_eq!(ret[1], 1);
        assert_eq!(ret[4], 0);
        let m3 = Floats::<i32, 24> {
            exponent: 1,
            significand: 65536,
        };
        let ret = m3.encode();
        assert_eq!(ret.len(), 4);
        assert_eq!(ret[0], 1);
        assert_eq!(ret[1], 1);
        assert_eq!(ret[3], 0);
        let m4 = Floats::<i32, 18> {
            exponent: 0,
            significand: -32767,
        };
        let ret = m4.encode();
        assert_eq!(ret.len(), 3);
        assert_eq!(ret[1], 128);
        assert_eq!(ret[2], 1);
        //test if signed bit has applied
        let m5 = Floats::<i32, 18> {
            exponent: 1,
            significand: -1,
        };
        let ret = m5.encode();
        assert_eq!(ret.len(), 3);
        assert_eq!(ret[0], 7);
        assert_eq!(ret[1], 255);
        assert_eq!(ret[2], 255);
    }

    #[test]
    fn test_decode() {
        let m1 = Float40 {
            exponent: 0,
            significand: 65536,
        };
        let ret = m1.encode();
        assert_eq!(Float40::decode(&ret).unwrap().significand, m1.significand);
        let m2 = Float40 {
            exponent: 1,
            significand: 16777216,
        };
        let ret = m2.encode();
        assert_eq!(Float40::decode(&ret).unwrap().significand, m2.significand);
        assert_eq!(Float40::decode(&ret).unwrap().exponent, m2.exponent);
        let m3 = Floats::<i32, 24> {
            exponent: 1,
            significand: 65536,
        };
        let ret = m3.encode();
        assert_eq!(
            Floats::<i32, 24>::decode(&ret).unwrap().significand,
            m3.significand
        );
        assert_eq!(
            Floats::<i32, 24>::decode(&ret).unwrap().exponent,
            m3.exponent
        );
        let m4 = Floats::<i32, 16> {
            exponent: 0,
            significand: -32767,
        };
        let ret = m4.encode();
        assert_eq!(
            Floats::<i32, 16>::decode(&ret).unwrap().significand,
            m4.significand
        );
        assert_eq!(
            Floats::<i32, 16>::decode(&ret).unwrap().exponent,
            m4.exponent
        );
        let m5 = Floats::<i32, 18> {
            exponent: 3,
            significand: -1,
        };
        let ret = m5.encode();
        assert_eq!(
            Floats::<i32, 18>::decode(&ret).unwrap().significand,
            m5.significand
        );
        assert_eq!(
            Floats::<i32, 18>::decode(&ret).unwrap().exponent,
            m5.exponent
        );
        let m6 = Floats::<u32, 16> {
            exponent: 50,
            significand: 65535,
        };
        let ret = m6.encode();
        assert_eq!(
            Floats::<u32, 16>::decode(&ret).unwrap().significand,
            m6.significand
        );
        assert_eq!(
            Floats::<u32, 16>::decode(&ret).unwrap().exponent,
            m6.exponent
        );
    }

    #[test]
    fn test_expression() {
        let m1 = Float40 {
            exponent: 0,
            significand: 65536,
        };
        assert_eq!(m1.to_bigint(), BigInt::from(65536));
        let m2 = Float40 {
            exponent: 1,
            significand: 16777216,
        };
        assert_eq!(m2.to_bigint(), BigInt::from(167772160u32));
        let m3 = Floats::<i32, 24> {
            exponent: 1,
            significand: 65536,
        };
        assert_eq!(m3.to_bigint(), BigInt::from(655360));
        let m4 = Floats::<i32, 16> {
            exponent: 0,
            significand: -32768,
        };
        assert_eq!(m4.to_bigint(), BigInt::from(-32768i32));
        let m5 = Floats::<i32, 18> {
            exponent: 2,
            significand: -32768,
        };
        assert_eq!(m5.to_bigint(), BigInt::from(-3276800i32));
        let m6 = Floats::<u32, 16> {
            exponent: 5,
            significand: 65535,
        };
        assert_eq!(m6.to_bigint(), BigInt::from(6553500000u64));
    }

    #[test]
    fn test_decimal() {
        let d = Decimal::new(1000, 0);
        let p1 = Float40::from_decimal(&d, 4).unwrap();
        assert_eq!(p1.exponent, 7);
        assert_eq!(p1.significand, 1);
        assert_eq!(d, p1.to_decimal(4));
        let d = Decimal::new(1000, 2);
        let p2 = Float40::from_decimal(&d, 4).unwrap();
        assert_eq!(p2.exponent, 5);
        assert_eq!(p2.significand, 1);
        assert_eq!(d, p2.to_decimal(4));
        let d = Decimal::new(1000000, 6);
        let p3 = Float40::from_decimal(&d, 4).unwrap();
        assert_eq!(p3.exponent, 4);
        assert_eq!(p3.significand, 1);
        assert_eq!(d, p3.to_decimal(4));
        let d = Decimal::new(12345678, 6);
        let p4 = Float40::from_decimal(&d, 8).unwrap();
        assert_eq!(p4.exponent, 2);
        assert_eq!(p4.significand, 12345678);
        assert_eq!(d, p4.to_decimal(8));
        let d = Decimal::new(1000000000000i64, 0);
        let p5 = Float40::from_decimal(&d, 2).unwrap();
        assert_eq!(p5.exponent, 14);
        assert_eq!(p5.significand, 1);
        assert_eq!(d, p5.to_decimal(2));
        let d = Decimal::new(123456, 5);
        let r5 = Floats::<u32, 16>::from_decimal(&d, 6).unwrap();
        assert_eq!(r5.exponent, 2);
        assert_eq!(r5.significand, 12345);
        let r5 = Floats::<i32, 16>::from_decimal(&d, 6).unwrap();
        assert_eq!(r5.exponent, 2);
        assert_eq!(r5.significand, 12345);

        let d = Decimal::new(-1000, 0);
        let m1 = Float40::from_decimal(&d, 4).unwrap();
        assert_eq!(m1.exponent, 7);
        assert_eq!(m1.significand, -1);
        assert_eq!(d, m1.to_decimal(4));
        let d = Decimal::new(-1000, 2);
        let m2 = Float40::from_decimal(&d, 4).unwrap();
        assert_eq!(m2.exponent, 5);
        assert_eq!(m2.significand, -1);
        assert_eq!(d, m2.to_decimal(4));
        let d = Decimal::new(-1000000, 6);
        let m3 = Float40::from_decimal(&d, 4).unwrap();
        assert_eq!(m3.exponent, 4);
        assert_eq!(m3.significand, -1);
        assert_eq!(d, m3.to_decimal(4));
        let d = Decimal::new(-12345678, 6);
        let m4 = Float40::from_decimal(&d, 8).unwrap();
        assert_eq!(m4.exponent, 2);
        assert_eq!(m4.significand, -12345678);
        assert_eq!(d, m4.to_decimal(8));
        let d = Decimal::new(-1000000000000i64, 0);
        let m5 = Float40::from_decimal(&d, 2).unwrap();
        assert_eq!(m5.exponent, 14);
        assert_eq!(m5.significand, -1);
        assert_eq!(d, m5.to_decimal(2));
        let d = Decimal::new(-123456, 5);
        let mr5 = Floats::<i32, 16>::from_decimal(&d, 6).unwrap();
        assert_eq!(mr5.exponent, 2);
        assert_eq!(mr5.significand, -12345);
    }

    #[test]
    fn test_edges() {
        // 1.23456 * 10**18

        let d0 = Decimal::new(0, 0);
        let f = Float40::from_decimal(&d0, 18).unwrap();
        assert_eq!(f.exponent, 0);
        assert_eq!(f.significand, 0);
        assert_eq!(f.to_bigint().to_u32().unwrap(), 0u32);
        println!("{}", f.to_encoded_int().unwrap());
        assert_eq!(f.to_encoded_int().unwrap().to_u32().unwrap(), 0u32);

        let f = Float40::from_encoded_bigint(BigInt::from(0)).unwrap();
        assert_eq!(f.exponent, 0);
        assert_eq!(f.significand, 0);
        assert_eq!(f.to_bigint().to_u32().unwrap(), 0u32);
        assert_eq!(f.to_encoded_int().unwrap().to_u32().unwrap(), 0u32);
    }

    #[test]
    fn test_float40() {
        // 1.23456 * 10**18

        let d0 = Decimal::new(123456, 5);
        let f = Float40::from_decimal(&d0, 18).unwrap();
        assert_eq!(f.exponent, 13);
        assert_eq!(f.significand, 123456);
        let d = f.to_decimal(18);
        assert_eq!(d, Decimal::from_str("1.23456").unwrap());
        let f2 = Float40::decode(&f.encode()).unwrap();
        assert_eq!(f2.exponent, 13);
        assert_eq!(f2.significand, 123456);
    }
}

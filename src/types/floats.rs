use std::convert::TryInto;

use super::{BigInt, Decimal};
use crate::num_traits::{identities::Zero, int::PrimInt, FromPrimitive, Pow, ToPrimitive};
use crate::types::{Fr, FrExt};

/// a POSTIVE float representation with 1 byte exponent and N bytes significand
#[derive(Debug, Clone, Copy)]
pub struct Floats<T: PrimInt, const N: usize> {
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
}

type Result<T, E = FloatsError> = std::result::Result<T, E>;

//it is caller's responsibility to ensure enough bytes for accommodating the decoded n
fn to_be_bytes<T: PrimInt + Zero>(n: T, out: &mut [u8]) {
    let mut n = n.swap_bytes();
    let mut i = 0;
    let mask = T::from(255u8).unwrap();
    while !n.is_zero() {
        out[i] = (n & mask).to_u8().unwrap();
        i += 1;
        n = n.unsigned_shr(8u32);
    }
}

impl<T: PrimInt + Zero, const N: usize> Floats<T, N> {
    pub fn to_bigint(self) -> BigInt {
        //cast to the largest int (128bit) possible
        let s = if T::min_value() < T::zero() {
            BigInt::from(self.significand.to_i128().unwrap())
        } else {
            BigInt::from(self.significand.to_u128().unwrap())
        };

        s * BigInt::from(10).pow(self.exponent)
    }

    pub fn to_fr(self) -> Fr {
        Fr::from_bigint(self.to_bigint())
    }

    pub fn encode(self) -> Vec<u8> {
        let bytes = (T::zero().count_zeros() / 8) as usize;
        assert!(bytes >= N);

        let mut result = vec![self.exponent];
        //use biggest buffer (16 bytes, 128bit)
        let mut buf = [0u8; 16];
        to_be_bytes(self.significand, &mut buf);
        let first_bit = buf[0] & 128;
        let used_buf = &mut buf[(bytes - N)..bytes];
        //resume the sign bit
        used_buf[0] |= first_bit;
        result.append(&mut used_buf.to_vec());
        result
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let bytes = (T::zero().count_zeros() / 8) as usize;
        assert!(bytes >= N && bytes <= 8);

        let exponent = u8::from_be_bytes(data.get(0..1).unwrap_or_default().try_into()?);
        let mut buf: [u8; 16] = if N == 16 {
            data.get(1..17).unwrap_or_default().try_into()?
        } else {
            [&[0u8; 16][0..(16 - N)], &data[1..]]
                .concat()
                .get(0..16)
                .unwrap_or_default()
                .try_into()?
        };
        let significand = if T::min_value() < T::zero() {
            //pick signal bit
            if buf[16 - N] & 128u8 != 0u8 {
                buf[0..16 - N].fill(255);
            }
            T::from(i128::from_be_bytes(buf)).unwrap()
        } else {
            T::from(u128::from_be_bytes(buf)).unwrap()
        };

        Ok(Self {
            exponent,
            significand,
        })
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

    //update from Decimal and round
    pub fn from_decimal(d: &Decimal, prec: u32) -> Result<Self> {
        let bytes = (T::zero().count_zeros() / 8) as usize;
        assert!(bytes >= N && bytes <= 8);

        //TODO: we are not able to handle T as u128 yet
        let test_low_bound = (T::min_value() >> ((bytes - N) * 8)).to_i128().unwrap();
        let test_high_bound = (T::max_value() >> ((bytes - N) * 8)).to_i128().unwrap();

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

pub type Float40 = Floats<i32, 4>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    //type Float864 = Floats<i64, 4>;

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
        assert_eq!(ret[0], 1);
        assert_eq!(ret[1], 1);
        assert_eq!(ret[4], 0);
        let m3 = Floats::<i32, 3> {
            exponent: 1,
            significand: 65536,
        };
        let ret = m3.encode();
        assert_eq!(ret.len(), 4);
        assert_eq!(ret[0], 1);
        assert_eq!(ret[1], 1);
        assert_eq!(ret[3], 0);
        let m4 = Floats::<i32, 2> {
            exponent: 0,
            significand: -32768,
        };
        let ret = m4.encode();
        assert_eq!(ret.len(), 3);
        assert_eq!(ret[1], 128);
        assert_eq!(ret[2], 0);
        //test if signed bit has applied
        let m4_overflowed = Floats::<i32, 2> {
            exponent: 0,
            significand: -65536,
        };
        let ret = m4_overflowed.encode();
        assert_eq!(ret.len(), 3);
        assert_eq!(ret[1], 128);
        assert_eq!(ret[2], 0);
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
        let m3 = Floats::<i32, 3> {
            exponent: 1,
            significand: 65536,
        };
        let ret = m3.encode();
        assert_eq!(
            Floats::<i32, 3>::decode(&ret).unwrap().significand,
            m3.significand
        );
        assert_eq!(
            Floats::<i32, 3>::decode(&ret).unwrap().exponent,
            m3.exponent
        );
        let m4 = Floats::<i32, 2> {
            exponent: 0,
            significand: -32768,
        };
        let ret = m4.encode();
        assert_eq!(
            Floats::<i32, 2>::decode(&ret).unwrap().significand,
            m4.significand
        );
        assert_eq!(
            Floats::<i32, 2>::decode(&ret).unwrap().exponent,
            m4.exponent
        );
        let m5 = Floats::<i32, 2> {
            exponent: 3,
            significand: -1,
        };
        let ret = m5.encode();
        assert_eq!(
            Floats::<i32, 2>::decode(&ret).unwrap().significand,
            m5.significand
        );
        assert_eq!(
            Floats::<i32, 2>::decode(&ret).unwrap().exponent,
            m5.exponent
        );
        let m6 = Floats::<u32, 2> {
            exponent: 50,
            significand: 65535,
        };
        let ret = m6.encode();
        assert_eq!(
            Floats::<u32, 2>::decode(&ret).unwrap().significand,
            m6.significand
        );
        assert_eq!(
            Floats::<u32, 2>::decode(&ret).unwrap().exponent,
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
        let m3 = Floats::<i32, 3> {
            exponent: 1,
            significand: 65536,
        };
        assert_eq!(m3.to_bigint(), BigInt::from(655360));
        let m4 = Floats::<i32, 2> {
            exponent: 0,
            significand: -32768,
        };
        assert_eq!(m4.to_bigint(), BigInt::from(-32768i32));
        let m5 = Floats::<i32, 2> {
            exponent: 2,
            significand: -32768,
        };
        assert_eq!(m5.to_bigint(), BigInt::from(-3276800i32));
        let m6 = Floats::<u32, 2> {
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
        let r5 = Floats::<u32, 2>::from_decimal(&d, 6).unwrap();
        assert_eq!(r5.exponent, 2);
        assert_eq!(r5.significand, 12345);
        let r5 = Floats::<i32, 2>::from_decimal(&d, 6).unwrap();
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
        let mr5 = Floats::<i32, 2>::from_decimal(&d, 6).unwrap();
        assert_eq!(mr5.exponent, 2);
        assert_eq!(mr5.significand, -12345);
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

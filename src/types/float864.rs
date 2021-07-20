use std::convert::TryInto;

use super::{BigInt, Decimal};
use crate::num_traits::Pow;
use crate::types::{Fr, FrExt};

/// a float representation with 1 byte exponent and 8 bytes significand
#[derive(Debug, Clone, Copy)]
pub struct Float864 {
    pub exponent: u8,
    // 5 bytes seems enough?
    pub significand: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum Float864Error {
    #[error("decimal precision error {0} {1}")]
    Precision(Decimal, u32),
    #[error("invalid precision {0} {1} {2}")]
    InvalidPrecision(Decimal, u32, Decimal),
    #[error(transparent)]
    TryFromSlice(#[from] std::array::TryFromSliceError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

type Result<T, E = Float864Error> = std::result::Result<T, E>;

impl Float864 {
    pub fn to_bigint(self) -> BigInt {
        let s = BigInt::from(self.significand);
        s * BigInt::from(10).pow(self.exponent)
    }

    pub fn to_fr(self) -> Fr {
        Fr::from_bigint(self.to_bigint())
    }

    pub fn encode(self) -> Vec<u8> {
        let mut result = self.exponent.to_be_bytes().to_vec();
        result.append(&mut self.significand.to_be_bytes().to_vec());
        result
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let exponent = u8::from_be_bytes(data[0..1].try_into()?);
        let significand = u64::from_be_bytes(data[1..9].try_into()?);
        Ok(Self {
            exponent,
            significand,
        })
    }

    pub fn to_decimal(self, prec: u32) -> Decimal {
        // for example, (significand:1, exponent:17) means 10**17, when prec is 18,
        // it is 0.1 (ETH)
        Decimal::new(self.significand as i64, 0) * Decimal::new(10, 0).pow(self.exponent as u64)
            / Decimal::new(10, 0).pow(prec as u64)
    }

    pub fn from_decimal(d: &Decimal, prec: u32) -> Result<Self> {
        // if d is "0.1" and prec is 18, result is (significand:1, exponent:17)
        if d.is_zero() {
            return Ok(Self {
                exponent: 0,
                significand: 0,
            });
        }
        let ten = Decimal::new(10, 0);
        let exp = ten.pow(prec as u64);
        let mut n = d * exp;
        if n != n.floor() {
            return Err(Float864Error::Precision(*d, prec));
        }
        let mut exponent = 0;
        loop {
            let next = n / ten;
            if next == next.floor() {
                exponent += 1;
                n = next;
            } else {
                break;
            }
        }
        if n > Decimal::new((u64::MAX / 4) as i64, 0) {
            return Err(Float864Error::InvalidPrecision(*d, prec, n));
        }
        // TODO: a better way...
        let significand: u64 = n.floor().to_string().parse::<u64>()?;
        Ok(Float864 {
            exponent,
            significand,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_float864() {
        // 1.23456 * 10**18
        let d0 = Decimal::new(123456, 5);
        let f = Float864::from_decimal(&d0, 18).unwrap();
        assert_eq!(f.exponent, 13);
        assert_eq!(f.significand, 123456);
        let d = f.to_decimal(18);
        assert_eq!(d, Decimal::from_str("1.23456").unwrap());
        let f2 = Float864::decode(&f.encode()).unwrap();
        assert_eq!(f2.exponent, 13);
        assert_eq!(f2.significand, 123456);
    }
}

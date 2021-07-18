use crate::Fr;
use crate::num_traits::{Pow, ToPrimitive};
use super::{Decimal, Float864, FrExt};

pub trait DecimalExt {
    fn to_u64(&self, prec: u32) -> u64;
    fn to_fr(&self, prec: u32) -> Fr;
    fn to_amount(&self, prec: u32) -> Float864;
}

impl DecimalExt for Decimal {
    fn to_u64(&self, prec: u32) -> u64 {
        let prec_mul = Decimal::new(10, 0).pow(prec as u64);
        let adjusted = self * prec_mul;
        ToPrimitive::to_u64(&adjusted.floor()).unwrap()
    }

    fn to_fr(&self, prec: u32) -> Fr {
        // TODO: is u64 enough?
        Fr::from_u64(DecimalExt::to_u64(self, prec))
        // Float864::from_decimal(num, prec).unwrap().to_fr()
    }

    fn to_amount(&self, prec: u32) -> Float864 {
        Float864::from_decimal(self, prec).unwrap()
    }
}
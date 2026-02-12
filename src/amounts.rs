use rust_decimal::{Decimal, RoundingStrategy};

use crate::types::asset::Asset;

/// Rounding strategy for BUY orders: round UP (away from zero).
pub const ROUNDING_BUY: RoundingStrategy = RoundingStrategy::AwayFromZero;
/// Rounding strategy for SELL orders: round DOWN (toward zero).
pub const ROUNDING_SELL: RoundingStrategy = RoundingStrategy::ToZero;
/// Rounding strategy for fees: round UP (away from zero).
pub const ROUNDING_FEE: RoundingStrategy = RoundingStrategy::AwayFromZero;

#[derive(Debug, Clone)]
pub struct HumanReadableAmount {
    pub value: Decimal,
    pub asset: Asset,
}

impl HumanReadableAmount {
    pub fn new(value: Decimal, asset: Asset) -> Self {
        Self { value, asset }
    }

    pub fn to_l1_amount(&self) -> L1Amount {
        let converted = self.asset.convert_internal_quantity_to_l1_quantity(self.value);
        L1Amount {
            value: converted,
            asset: self.asset.clone(),
        }
    }

    pub fn to_stark_amount(&self, rounding: RoundingStrategy) -> StarkAmount {
        let converted = self
            .asset
            .convert_human_readable_to_stark_quantity(self.value, rounding);
        StarkAmount {
            value: converted,
            asset: self.asset.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct L1Amount {
    pub value: i64,
    pub asset: Asset,
}

impl L1Amount {
    pub fn to_internal_amount(&self) -> HumanReadableAmount {
        let converted = self.asset.convert_l1_quantity_to_internal_quantity(self.value);
        HumanReadableAmount {
            value: converted,
            asset: self.asset.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StarkAmount {
    pub value: i64,
    pub asset: Asset,
}

impl StarkAmount {
    pub fn to_internal_amount(&self) -> HumanReadableAmount {
        let converted = self.asset.convert_stark_to_internal_quantity(self.value);
        HumanReadableAmount {
            value: converted,
            asset: self.asset.clone(),
        }
    }

    pub fn negate(&self) -> StarkAmount {
        StarkAmount {
            value: -self.value,
            asset: self.asset.clone(),
        }
    }
}

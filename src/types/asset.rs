use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::common::HexValue;
use crate::types::enums::{AssetOperationStatus, AssetOperationType};

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub precision: u32,
    pub active: bool,
    pub is_collateral: bool,
    pub settlement_external_id: String,
    pub settlement_resolution: i64,
    pub l1_external_id: String,
    pub l1_resolution: i64,
}

impl Asset {
    pub fn convert_human_readable_to_stark_quantity(
        &self,
        internal: Decimal,
        strategy: rust_decimal::RoundingStrategy,
    ) -> i64 {
        let scaled = internal * Decimal::from(self.settlement_resolution);
        scaled
            .round_dp_with_strategy(0, strategy)
            .to_string()
            .parse::<i64>()
            .expect("stark quantity conversion overflow")
    }

    pub fn convert_stark_to_internal_quantity(&self, stark: i64) -> Decimal {
        Decimal::from(stark) / Decimal::from(self.settlement_resolution)
    }

    pub fn convert_l1_quantity_to_internal_quantity(&self, l1: i64) -> Decimal {
        Decimal::from(l1) / Decimal::from(self.l1_resolution)
    }

    pub fn convert_internal_quantity_to_l1_quantity(&self, internal: Decimal) -> i64 {
        assert!(self.is_collateral, "Only collateral assets have L1 representation");
        let scaled = internal * Decimal::from(self.l1_resolution);
        scaled
            .to_string()
            .parse::<i64>()
            .expect("l1 quantity conversion overflow")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetOperationModel {
    pub id: String,
    #[serde(rename = "type")]
    pub operation_type: AssetOperationType,
    pub status: AssetOperationStatus,
    pub amount: Decimal,
    pub fee: Decimal,
    pub asset: i64,
    pub time: i64,
    pub account_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterparty_account_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<HexValue>,
}

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::enums::{ExitType, PositionSide, PositionStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionModel {
    pub id: i64,
    pub account_id: i64,
    pub market: String,
    pub status: PositionStatus,
    pub side: PositionSide,
    pub leverage: Decimal,
    pub size: Decimal,
    pub value: Decimal,
    pub open_price: Decimal,
    pub mark_price: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidation_price: Option<Decimal>,
    pub unrealised_pnl: Decimal,
    pub realised_pnl: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tp_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sl_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adl: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealisedPnlBreakdownModel {
    pub trade_pnl: Decimal,
    pub funding_fees: Decimal,
    pub open_fees: Decimal,
    pub close_fees: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionHistoryModel {
    pub id: i64,
    pub account_id: i64,
    pub market: String,
    pub side: PositionSide,
    pub size: Decimal,
    pub max_position_size: Decimal,
    pub leverage: Decimal,
    pub open_price: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_price: Option<Decimal>,
    pub realised_pnl: Decimal,
    pub realised_pnl_breakdown: RealisedPnlBreakdownModel,
    pub created_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_type: Option<ExitType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_time: Option<i64>,
}

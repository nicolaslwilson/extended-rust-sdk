use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::common::{HexValue, SettlementSignatureModel};
use crate::types::enums::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarkSettlementModel {
    pub signature: SettlementSignatureModel,
    pub stark_key: HexValue,
    pub collateral_position: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarkDebuggingOrderAmountsModel {
    pub collateral_amount: Decimal,
    pub fee_amount: Decimal,
    pub synthetic_amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderConditionalTriggerModel {
    pub trigger_price: Decimal,
    pub trigger_price_type: OrderTriggerPriceType,
    pub direction: OrderTriggerDirection,
    pub execution_price_type: OrderPriceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderTpslTriggerModel {
    pub trigger_price: Decimal,
    pub trigger_price_type: OrderTriggerPriceType,
    pub price: Decimal,
    pub price_type: OrderPriceType,
    pub settlement: StarkSettlementModel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debugging_amounts: Option<StarkDebuggingOrderAmountsModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderModel {
    pub id: String,
    pub market: String,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub qty: Decimal,
    pub price: Decimal,
    #[serde(default)]
    pub reduce_only: bool,
    #[serde(default)]
    pub post_only: bool,
    pub time_in_force: TimeInForce,
    pub expiry_epoch_millis: i64,
    pub fee: Decimal,
    pub nonce: Decimal,
    pub self_trade_protection_level: SelfTradeProtectionLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement: Option<StarkSettlementModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<CreateOrderConditionalTriggerModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tp_sl_type: Option<OrderTpslType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<CreateOrderTpslTriggerModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<CreateOrderTpslTriggerModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debugging_amounts: Option<StarkDebuggingOrderAmountsModel>,
    #[serde(rename = "builderFee", skip_serializing_if = "Option::is_none")]
    pub builder_fee: Option<Decimal>,
    #[serde(rename = "builderId", skip_serializing_if = "Option::is_none")]
    pub builder_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedOrderModel {
    pub id: i64,
    pub external_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrderTpslTriggerModel {
    pub trigger_price: Decimal,
    pub trigger_price_type: OrderTriggerPriceType,
    pub price: Decimal,
    pub price_type: OrderPriceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrderModel {
    pub id: i64,
    pub account_id: i64,
    pub external_id: String,
    pub market: String,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub status: OrderStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<OrderStatusReason>,
    pub price: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_price: Option<Decimal>,
    pub qty: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled_qty: Option<Decimal>,
    pub reduce_only: bool,
    pub post_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payed_fee: Option<Decimal>,
    pub created_time: i64,
    pub updated_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_time: Option<i64>,
    pub time_in_force: TimeInForce,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tp_sl_type: Option<OrderTpslType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<OpenOrderTpslTriggerModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<OpenOrderTpslTriggerModel>,
}

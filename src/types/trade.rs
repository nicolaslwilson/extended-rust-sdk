use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::enums::{OrderSide, TradeType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicTradeModel {
    #[serde(alias = "i")]
    pub id: i64,
    #[serde(alias = "m")]
    pub market: String,
    #[serde(alias = "S")]
    pub side: OrderSide,
    #[serde(alias = "tT", rename = "tT")]
    pub trade_type: TradeType,
    #[serde(alias = "T")]
    pub timestamp: i64,
    #[serde(alias = "p")]
    pub price: Decimal,
    #[serde(alias = "q")]
    pub qty: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountTradeModel {
    pub id: i64,
    pub account_id: i64,
    pub market: String,
    pub order_id: i64,
    pub side: OrderSide,
    pub price: Decimal,
    pub qty: Decimal,
    pub value: Decimal,
    pub fee: Decimal,
    pub is_taker: bool,
    pub trade_type: TradeType,
    pub created_time: i64,
}

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::common::{HexValue, SettlementSignatureModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp {
    pub seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarkWithdrawalSettlement {
    pub recipient: HexValue,
    pub position_id: u64,
    pub collateral_id: HexValue,
    pub amount: i64,
    pub expiration: Timestamp,
    pub salt: u32,
    pub signature: SettlementSignatureModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalRequest {
    pub account_id: i64,
    pub amount: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub settlement: StarkWithdrawalSettlement,
    pub chain_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
    pub asset: String,
}

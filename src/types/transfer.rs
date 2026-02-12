use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::common::{HexValue, SettlementSignatureModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarkTransferSettlement {
    pub amount: i64,
    pub asset_id: HexValue,
    pub expiration_timestamp: i64,
    pub nonce: u32,
    pub receiver_position_id: u64,
    pub receiver_public_key: HexValue,
    pub sender_position_id: u64,
    pub sender_public_key: HexValue,
    pub signature: SettlementSignatureModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnChainPerpetualTransferModel {
    pub from_vault: u64,
    pub to_vault: u64,
    pub amount: Decimal,
    pub settlement: StarkTransferSettlement,
    pub transferred_asset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResponseModel {
    pub valid_signature: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_calculated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stark_ex_representation: Option<serde_json::Value>,
}

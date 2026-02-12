use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientModel {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starknet_wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral_link_code: Option<String>,
}

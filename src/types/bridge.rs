use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainConfig {
    pub chain: String,
    pub contract_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgesConfig {
    pub chains: Vec<ChainConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: String,
    pub fee: Decimal,
}

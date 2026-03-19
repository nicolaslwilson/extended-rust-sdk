use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use starknet_crypto::Felt;

use crate::types::balance::BalanceModel;
use crate::types::common::deserialize_i64_from_string_or_number;
use crate::types::fee::TradingFeeModel;
use crate::types::order::OpenOrderModel;
use crate::types::position::PositionModel;
use crate::types::trade::AccountTradeModel;

/// Stark perpetual account used for signing orders and transactions.
#[derive(Debug, Clone)]
pub struct StarkPerpetualAccount {
    vault: u64,
    private_key: Felt,
    public_key: Felt,
    api_key: String,
    trading_fee: HashMap<String, TradingFeeModel>,
}

impl StarkPerpetualAccount {
    pub fn new(vault: u64, private_key: &str, public_key: &str, api_key: &str) -> Self {
        let private_key = Felt::from_hex(private_key).expect("invalid private key hex");
        let public_key = Felt::from_hex(public_key).expect("invalid public key hex");

        Self {
            vault,
            private_key,
            public_key,
            api_key: api_key.to_string(),
            trading_fee: HashMap::new(),
        }
    }

    pub fn vault(&self) -> u64 {
        self.vault
    }

    pub fn public_key(&self) -> Felt {
        self.public_key
    }

    pub fn public_key_hex(&self) -> String {
        format!("{:#x}", self.public_key)
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn trading_fee(&self) -> &HashMap<String, TradingFeeModel> {
        &self.trading_fee
    }

    pub fn set_trading_fee(&mut self, fees: HashMap<String, TradingFeeModel>) {
        self.trading_fee = fees;
    }

    pub fn sign(&self, msg_hash: Felt) -> (Felt, Felt) {
        let sig = rust_crypto_lib_base::sign_message(&msg_hash, &self.private_key)
            .expect("signing failed");
        (sig.r, sig.s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountModel {
    #[serde(
        alias = "accountId",
        deserialize_with = "deserialize_i64_from_string_or_number"
    )]
    pub id: i64,
    pub description: String,
    #[serde(deserialize_with = "deserialize_i64_from_string_or_number")]
    pub account_index: i64,
    pub status: String,
    pub l2_key: String,
    #[serde(deserialize_with = "deserialize_i64_from_string_or_number")]
    pub l2_vault: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge_starknet_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountLeverage {
    pub market: String,
    pub leverage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountStreamDataModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<OpenOrderModel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<Vec<PositionModel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<Vec<AccountTradeModel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<BalanceModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponseModel {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRequestModel {
    pub description: String,
}

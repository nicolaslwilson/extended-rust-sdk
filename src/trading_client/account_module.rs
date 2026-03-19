use std::collections::HashMap;
use std::sync::Arc;

use rust_decimal::Decimal;
use serde_json::Value;
use starknet_crypto::Felt;

use crate::error::{X10Error, X10Result};
use crate::http::client::HttpClient;
use crate::http::response::WrappedApiResponse;
use crate::transfer_builder::create_transfer_object;
use crate::types::account::{AccountLeverage, AccountModel, StarkPerpetualAccount};
use crate::types::asset::AssetOperationModel;
use crate::types::balance::BalanceModel;
use crate::types::bridge::{BridgesConfig, Quote};
use crate::types::client::ClientModel;
use crate::types::config::EndpointConfig;
use crate::types::enums::{
    AssetOperationStatus, AssetOperationType, OrderSide, OrderType, PositionSide, TradeType,
};
use crate::types::fee::TradingFeeModel;
use crate::types::order::OpenOrderModel;
use crate::types::position::{PositionHistoryModel, PositionModel};
use crate::types::trade::AccountTradeModel;
use crate::types::transfer::TransferResponseModel;
use crate::withdrawal_builder::create_withdrawal_object;

use super::url_builder::{build_url, QueryValue};

pub struct AccountModule {
    http: Arc<HttpClient>,
    base_url: String,
    config: EndpointConfig,
    stark_account: Option<StarkPerpetualAccount>,
}

impl AccountModule {
    pub(crate) fn new(
        http: Arc<HttpClient>,
        base_url: String,
        config: EndpointConfig,
        stark_account: Option<StarkPerpetualAccount>,
    ) -> Self {
        Self {
            http,
            base_url,
            config,
            stark_account,
        }
    }

    fn account(&self) -> X10Result<&StarkPerpetualAccount> {
        self.stark_account
            .as_ref()
            .ok_or(X10Error::StarkAccountNotSet)
    }

    pub async fn get_account(&self) -> X10Result<WrappedApiResponse<AccountModel>> {
        let url = build_url(&self.base_url, "/user/account/info", &HashMap::new(), &[]);
        self.http.get(&url, None).await
    }

    pub async fn get_client(&self) -> X10Result<WrappedApiResponse<ClientModel>> {
        let url = build_url(&self.base_url, "/user/client/info", &HashMap::new(), &[]);
        self.http.get(&url, None).await
    }

    pub async fn get_balance(&self) -> X10Result<WrappedApiResponse<BalanceModel>> {
        let url = build_url(&self.base_url, "/user/balance", &HashMap::new(), &[]);
        self.http.get(&url, None).await
    }

    pub async fn get_positions(
        &self,
        market_names: Option<&[String]>,
        position_side: Option<PositionSide>,
    ) -> X10Result<WrappedApiResponse<Vec<PositionModel>>> {
        let url = build_url(
            &self.base_url,
            "/user/positions",
            &HashMap::new(),
            &[
                ("market", QueryValue::from_opt_list(market_names)),
                (
                    "side",
                    position_side
                        .map(|s| QueryValue::Single(s.to_string()))
                        .unwrap_or(QueryValue::None),
                ),
            ],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_positions_history(
        &self,
        market_names: Option<&[String]>,
        position_side: Option<PositionSide>,
        cursor: Option<i64>,
        limit: Option<i64>,
    ) -> X10Result<WrappedApiResponse<Vec<PositionHistoryModel>>> {
        let url = build_url(
            &self.base_url,
            "/user/positions/history",
            &HashMap::new(),
            &[
                ("market", QueryValue::from_opt_list(market_names)),
                (
                    "side",
                    position_side
                        .map(|s| QueryValue::Single(s.to_string()))
                        .unwrap_or(QueryValue::None),
                ),
                ("cursor", QueryValue::from_opt_i64(cursor)),
                ("limit", QueryValue::from_opt_i64(limit)),
            ],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_open_orders(
        &self,
        market_names: Option<&[String]>,
        order_type: Option<OrderType>,
        order_side: Option<OrderSide>,
    ) -> X10Result<WrappedApiResponse<Vec<OpenOrderModel>>> {
        let url = build_url(
            &self.base_url,
            "/user/orders",
            &HashMap::new(),
            &[
                ("market", QueryValue::from_opt_list(market_names)),
                (
                    "type",
                    order_type
                        .map(|t| QueryValue::Single(t.to_string()))
                        .unwrap_or(QueryValue::None),
                ),
                (
                    "side",
                    order_side
                        .map(|s| QueryValue::Single(s.to_string()))
                        .unwrap_or(QueryValue::None),
                ),
            ],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_orders_history(
        &self,
        market_names: Option<&[String]>,
        order_type: Option<OrderType>,
        order_side: Option<OrderSide>,
        cursor: Option<i64>,
        limit: Option<i64>,
    ) -> X10Result<WrappedApiResponse<Vec<OpenOrderModel>>> {
        let url = build_url(
            &self.base_url,
            "/user/orders/history",
            &HashMap::new(),
            &[
                ("market", QueryValue::from_opt_list(market_names)),
                (
                    "type",
                    order_type
                        .map(|t| QueryValue::Single(t.to_string()))
                        .unwrap_or(QueryValue::None),
                ),
                (
                    "side",
                    order_side
                        .map(|s| QueryValue::Single(s.to_string()))
                        .unwrap_or(QueryValue::None),
                ),
                ("cursor", QueryValue::from_opt_i64(cursor)),
                ("limit", QueryValue::from_opt_i64(limit)),
            ],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_order_by_id(
        &self,
        order_id: i64,
    ) -> X10Result<WrappedApiResponse<OpenOrderModel>> {
        let mut params = HashMap::new();
        params.insert("order_id", order_id.to_string());
        let url = build_url(&self.base_url, "/user/orders/<order_id>", &params, &[]);
        self.http.get(&url, None).await
    }

    pub async fn get_order_by_external_id(
        &self,
        external_id: &str,
    ) -> X10Result<WrappedApiResponse<Vec<OpenOrderModel>>> {
        let mut params = HashMap::new();
        params.insert("external_id", external_id.to_string());
        let url = build_url(
            &self.base_url,
            "/user/orders/external/<external_id>",
            &params,
            &[],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_trades(
        &self,
        market_names: Option<&[String]>,
        trade_side: Option<OrderSide>,
        trade_type: Option<TradeType>,
        cursor: Option<i64>,
        limit: Option<i64>,
    ) -> X10Result<WrappedApiResponse<Vec<AccountTradeModel>>> {
        let url = build_url(
            &self.base_url,
            "/user/trades",
            &HashMap::new(),
            &[
                ("market", QueryValue::from_opt_list(market_names)),
                (
                    "side",
                    trade_side
                        .map(|s| QueryValue::Single(s.to_string()))
                        .unwrap_or(QueryValue::None),
                ),
                (
                    "type",
                    trade_type
                        .map(|t| QueryValue::Single(t.to_string()))
                        .unwrap_or(QueryValue::None),
                ),
                ("cursor", QueryValue::from_opt_i64(cursor)),
                ("limit", QueryValue::from_opt_i64(limit)),
            ],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_fees(
        &self,
        market_names: Option<&[String]>,
        builder_id: Option<i64>,
    ) -> X10Result<WrappedApiResponse<Vec<TradingFeeModel>>> {
        let url = build_url(
            &self.base_url,
            "/user/fees",
            &HashMap::new(),
            &[
                ("market", QueryValue::from_opt_list(market_names)),
                ("builderId", QueryValue::from_opt_i64(builder_id)),
            ],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_leverage(
        &self,
        market_names: Option<&[String]>,
    ) -> X10Result<WrappedApiResponse<Vec<AccountLeverage>>> {
        let url = build_url(
            &self.base_url,
            "/user/leverage",
            &HashMap::new(),
            &[("market", QueryValue::from_opt_list(market_names))],
        );
        self.http.get(&url, None).await
    }

    pub async fn update_leverage(
        &self,
        market_name: &str,
        leverage: Decimal,
    ) -> X10Result<WrappedApiResponse<Value>> {
        let url = build_url(&self.base_url, "/user/leverage", &HashMap::new(), &[]);
        let body = AccountLeverage {
            market: market_name.to_string(),
            leverage,
        };
        self.http.patch(&url, Some(&body), None).await
    }

    pub async fn get_bridge_config(&self) -> X10Result<WrappedApiResponse<BridgesConfig>> {
        let url = build_url(&self.base_url, "/user/bridge/config", &HashMap::new(), &[]);
        self.http.get(&url, None).await
    }

    pub async fn get_bridge_quote(
        &self,
        chain_in: &str,
        chain_out: &str,
        amount: Decimal,
    ) -> X10Result<WrappedApiResponse<Quote>> {
        let url = build_url(
            &self.base_url,
            "/user/bridge/quote",
            &HashMap::new(),
            &[
                ("chainIn", QueryValue::Single(chain_in.to_string())),
                ("chainOut", QueryValue::Single(chain_out.to_string())),
                ("amount", QueryValue::Single(amount.to_string())),
            ],
        );
        self.http.get(&url, None).await
    }

    pub async fn commit_bridge_quote(&self, id: &str) -> X10Result<WrappedApiResponse<Value>> {
        let url = build_url(
            &self.base_url,
            "/user/bridge/quote",
            &HashMap::new(),
            &[("id", QueryValue::Single(id.to_string()))],
        );
        self.http.post::<Value, Value>(&url, None, None).await
    }

    pub async fn transfer(
        &self,
        to_vault: u64,
        to_l2_key: &str,
        amount: Decimal,
        nonce: Option<u32>,
    ) -> X10Result<WrappedApiResponse<TransferResponseModel>> {
        let account = self.account()?;
        let from_vault = account.vault();
        let to_l2_key_felt = Felt::from_hex(to_l2_key)
            .map_err(|e| X10Error::Validation(format!("invalid to_l2_key: {}", e)))?;

        let request_model = create_transfer_object(
            from_vault,
            to_vault,
            to_l2_key_felt,
            amount,
            &self.config,
            account,
            nonce,
        )
        .map_err(X10Error::Crypto)?;

        let url = build_url(
            &self.base_url,
            "/user/transfer/onchain",
            &HashMap::new(),
            &[],
        );
        let body = serde_json::to_value(&request_model)?;
        self.http.post(&url, Some(&body), None).await
    }

    pub async fn withdraw(
        &self,
        amount: Decimal,
        chain_id: &str,
        stark_address: Option<&str>,
        nonce: Option<u32>,
        quote_id: Option<String>,
    ) -> X10Result<WrappedApiResponse<Value>> {
        let account = self.account()?;

        if quote_id.is_none() && chain_id != "STRK" {
            return Err(X10Error::Validation(
                "quote_id is required for EVM withdrawals".into(),
            ));
        }

        let acct_model = self.get_account().await?;
        let acct = acct_model
            .data
            .ok_or_else(|| X10Error::Other("Account not found".into()))?;

        let recipient_stark_address = if let Some(addr) = stark_address {
            addr.to_string()
        } else if chain_id == "STRK" {
            let client_resp = self.get_client().await?;
            let client = client_resp
                .data
                .ok_or_else(|| X10Error::Other("Client not found".into()))?;
            client.starknet_wallet_address.ok_or_else(|| {
                X10Error::Validation("Client does not have attached starknet_wallet_address".into())
            })?
        } else {
            acct.bridge_starknet_address.ok_or_else(|| {
                X10Error::Validation("Account bridge_starknet_address not found".into())
            })?
        };

        let request_model = create_withdrawal_object(
            amount,
            &recipient_stark_address,
            account,
            &self.config,
            acct.id,
            chain_id,
            None,
            nonce,
            quote_id,
        )
        .map_err(X10Error::Crypto)?;

        let url = build_url(&self.base_url, "/user/withdrawal", &HashMap::new(), &[]);
        let body = serde_json::to_value(&request_model)?;
        self.http.post(&url, Some(&body), None).await
    }

    pub async fn asset_operations(
        &self,
        id: Option<i64>,
        operations_type: Option<&[AssetOperationType]>,
        operations_status: Option<&[AssetOperationStatus]>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        cursor: Option<i64>,
        limit: Option<i64>,
    ) -> X10Result<WrappedApiResponse<Vec<AssetOperationModel>>> {
        let type_strs: Option<Vec<String>> =
            operations_type.map(|types| types.iter().map(|t| t.to_string()).collect());
        let status_strs: Option<Vec<String>> =
            operations_status.map(|statuses| statuses.iter().map(|s| s.to_string()).collect());

        let url = build_url(
            &self.base_url,
            "/user/assetOperations",
            &HashMap::new(),
            &[
                (
                    "type",
                    type_strs
                        .as_deref()
                        .map(|s| QueryValue::List(s.to_vec()))
                        .unwrap_or(QueryValue::None),
                ),
                (
                    "status",
                    status_strs
                        .as_deref()
                        .map(|s| QueryValue::List(s.to_vec()))
                        .unwrap_or(QueryValue::None),
                ),
                ("startTime", QueryValue::from_opt_i64(start_time)),
                ("endTime", QueryValue::from_opt_i64(end_time)),
                ("cursor", QueryValue::from_opt_i64(cursor)),
                ("limit", QueryValue::from_opt_i64(limit)),
                ("id", QueryValue::from_opt_i64(id)),
            ],
        );
        self.http.get(&url, None).await
    }
}

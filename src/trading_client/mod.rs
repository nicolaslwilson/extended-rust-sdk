mod account_module;
mod info_module;
mod markets_module;
mod order_management;
mod testnet_module;
pub(crate) mod url_builder;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use tokio::sync::Mutex;

use crate::error::{X10Error, X10Result};
use crate::http::client::HttpClient;
use crate::http::response::WrappedApiResponse;
use crate::order_builder::{create_order_object, OrderTpslTriggerParam};
use crate::types::account::StarkPerpetualAccount;
use crate::types::config::EndpointConfig;
use crate::types::enums::*;
use crate::types::market::MarketModel;
use crate::types::order::PlacedOrderModel;
use crate::utils::time::utc_now;

pub use account_module::AccountModule;
pub use info_module::InfoModule;
pub use markets_module::MarketsInformationModule;
pub use order_management::OrderManagementModule;
pub use testnet_module::TestnetModule;

/// Main trading client for the Extended exchange REST API.
pub struct PerpetualTradingClient {
    config: EndpointConfig,
    stark_account: Option<StarkPerpetualAccount>,
    markets_cache: Arc<Mutex<Option<HashMap<String, MarketModel>>>>,
    info: InfoModule,
    markets_info: MarketsInformationModule,
    account: AccountModule,
    orders: OrderManagementModule,
    testnet: TestnetModule,
}

impl PerpetualTradingClient {
    pub fn new(
        endpoint_config: EndpointConfig,
        stark_account: Option<StarkPerpetualAccount>,
    ) -> X10Result<Self> {
        let api_key = stark_account.as_ref().map(|a| a.api_key().to_string());

        let http_client = Arc::new(HttpClient::new(api_key)?);
        let base_url = endpoint_config.api_base_url.clone();

        Ok(Self {
            info: InfoModule::new(http_client.clone(), base_url.clone()),
            markets_info: MarketsInformationModule::new(http_client.clone(), base_url.clone()),
            account: AccountModule::new(
                http_client.clone(),
                base_url.clone(),
                endpoint_config.clone(),
                stark_account.clone(),
            ),
            orders: OrderManagementModule::new(http_client.clone(), base_url.clone()),
            testnet: TestnetModule::new(http_client.clone(), base_url.clone()),
            config: endpoint_config,
            stark_account,
            markets_cache: Arc::new(Mutex::new(None)),
        })
    }

    pub fn info(&self) -> &InfoModule {
        &self.info
    }

    pub fn markets_info(&self) -> &MarketsInformationModule {
        &self.markets_info
    }

    pub fn account(&self) -> &AccountModule {
        &self.account
    }

    pub fn orders(&self) -> &OrderManagementModule {
        &self.orders
    }

    pub fn testnet(&self) -> &TestnetModule {
        &self.testnet
    }

    /// Convenience method: auto-fetches market, builds + signs + places order.
    #[allow(clippy::too_many_arguments)]
    pub async fn place_order(
        &self,
        market_name: &str,
        amount_of_synthetic: Decimal,
        price: Decimal,
        side: OrderSide,
        post_only: bool,
        previous_order_id: Option<String>,
        expire_time: Option<DateTime<Utc>>,
        time_in_force: TimeInForce,
        self_trade_protection_level: SelfTradeProtectionLevel,
        external_id: Option<String>,
        builder_fee: Option<Decimal>,
        builder_id: Option<i64>,
        reduce_only: bool,
        tp_sl_type: Option<OrderTpslType>,
        take_profit: Option<OrderTpslTriggerParam>,
        stop_loss: Option<OrderTpslTriggerParam>,
    ) -> X10Result<WrappedApiResponse<PlacedOrderModel>> {
        let account = self
            .stark_account
            .as_ref()
            .ok_or(X10Error::StarkAccountNotSet)?;

        // Lazy-load markets
        {
            let mut cache = self.markets_cache.lock().await;
            if cache.is_none() {
                let markets_response = self.markets_info.get_markets(None).await?;
                if let Some(markets) = markets_response.data {
                    let map: HashMap<String, MarketModel> =
                        markets.into_iter().map(|m| (m.name.clone(), m)).collect();
                    *cache = Some(map);
                }
            }
        }

        let cache = self.markets_cache.lock().await;
        let markets = cache.as_ref().ok_or_else(|| {
            X10Error::Other("Failed to fetch markets".into())
        })?;

        let market = markets.get(market_name).ok_or_else(|| {
            X10Error::Validation(format!("Market {} not found", market_name))
        })?;

        let expire_time = expire_time.unwrap_or_else(|| utc_now() + Duration::hours(1));

        let order = create_order_object(
            account,
            market,
            amount_of_synthetic,
            price,
            side,
            &self.config.starknet_domain,
            post_only,
            previous_order_id,
            Some(expire_time),
            external_id,
            time_in_force,
            self_trade_protection_level,
            None,
            builder_fee,
            builder_id,
            reduce_only,
            tp_sl_type,
            take_profit,
            stop_loss,
        )?;

        self.orders.place_order(&order).await
    }
}

mod connection;

use std::collections::HashMap;

use crate::http::response::WrappedStreamResponse;
use crate::trading_client::url_builder::{build_url, QueryValue};
use crate::types::account::AccountStreamDataModel;
use crate::types::candle::CandleModel;
use crate::types::enums::{CandleInterval, CandleType};
use crate::types::funding::FundingRateModel;
use crate::types::orderbook::OrderbookUpdateModel;
use crate::types::trade::PublicTradeModel;

pub use connection::StreamConnection;

/// WebSocket stream client for the Extended exchange.
pub struct PerpetualStreamClient {
    api_url: String,
}

impl PerpetualStreamClient {
    pub fn new(api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
        }
    }

    /// Subscribe to orderbook updates.
    pub async fn subscribe_to_orderbooks(
        &self,
        market_name: Option<&str>,
        depth: Option<i64>,
    ) -> crate::error::X10Result<StreamConnection<WrappedStreamResponse<OrderbookUpdateModel>>>
    {
        let mut params = HashMap::new();
        if let Some(market) = market_name {
            params.insert("market", market.to_string());
        }
        let mut query = Vec::new();
        if let Some(d) = depth {
            query.push(("depth", QueryValue::Single(d.to_string())));
        }
        let url = build_url(&self.api_url, "/orderbooks/<market?>", &params, &query);
        StreamConnection::connect(&url, None).await
    }

    /// Subscribe to public trades.
    pub async fn subscribe_to_public_trades(
        &self,
        market_name: Option<&str>,
    ) -> crate::error::X10Result<StreamConnection<WrappedStreamResponse<Vec<PublicTradeModel>>>>
    {
        let mut params = HashMap::new();
        if let Some(market) = market_name {
            params.insert("market", market.to_string());
        }
        let url = build_url(&self.api_url, "/publicTrades/<market?>", &params, &[]);
        StreamConnection::connect(&url, None).await
    }

    /// Subscribe to funding rate updates.
    pub async fn subscribe_to_funding_rates(
        &self,
        market_name: Option<&str>,
    ) -> crate::error::X10Result<StreamConnection<WrappedStreamResponse<FundingRateModel>>>
    {
        let mut params = HashMap::new();
        if let Some(market) = market_name {
            params.insert("market", market.to_string());
        }
        let url = build_url(&self.api_url, "/funding/<market?>", &params, &[]);
        StreamConnection::connect(&url, None).await
    }

    /// Subscribe to candle updates.
    pub async fn subscribe_to_candles(
        &self,
        market_name: &str,
        candle_type: CandleType,
        interval: CandleInterval,
    ) -> crate::error::X10Result<StreamConnection<WrappedStreamResponse<Vec<CandleModel>>>>
    {
        let mut params = HashMap::new();
        params.insert("market", market_name.to_string());
        params.insert("candle_type", candle_type.to_string());
        let url = build_url(
            &self.api_url,
            "/candles/<market>/<candle_type>",
            &params,
            &[("interval", QueryValue::Single(interval.to_string()))],
        );
        StreamConnection::connect(&url, None).await
    }

    /// Subscribe to account updates (authenticated).
    pub async fn subscribe_to_account_updates(
        &self,
        api_key: &str,
    ) -> crate::error::X10Result<StreamConnection<WrappedStreamResponse<AccountStreamDataModel>>>
    {
        let url = build_url(&self.api_url, "/account", &HashMap::new(), &[]);
        StreamConnection::connect(&url, Some(api_key)).await
    }
}

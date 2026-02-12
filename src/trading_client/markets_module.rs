use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::error::X10Result;
use crate::http::client::HttpClient;
use crate::http::response::WrappedApiResponse;
use crate::types::candle::CandleModel;
use crate::types::enums::{CandleInterval, CandleType};
use crate::types::funding::FundingRateModel;
use crate::types::market::{MarketModel, MarketStatsModel};
use crate::types::orderbook::OrderbookUpdateModel;
use crate::utils::time::to_epoch_millis;

use super::url_builder::{build_url, QueryValue};

pub struct MarketsInformationModule {
    http: Arc<HttpClient>,
    base_url: String,
}

impl MarketsInformationModule {
    pub(crate) fn new(http: Arc<HttpClient>, base_url: String) -> Self {
        Self { http, base_url }
    }

    pub async fn get_markets(
        &self,
        market_names: Option<&[String]>,
    ) -> X10Result<WrappedApiResponse<Vec<MarketModel>>> {
        let url = build_url(
            &self.base_url,
            "/info/markets",
            &HashMap::new(),
            &[("market", QueryValue::from_opt_list(market_names))],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_markets_dict(&self) -> X10Result<HashMap<String, MarketModel>> {
        let resp = self.get_markets(None).await?;
        let markets = resp.data.unwrap_or_default();
        Ok(markets.into_iter().map(|m| (m.name.clone(), m)).collect())
    }

    pub async fn get_market_statistics(
        &self,
        market_name: &str,
    ) -> X10Result<WrappedApiResponse<MarketStatsModel>> {
        let mut params = HashMap::new();
        params.insert("market", market_name.to_string());
        let url = build_url(
            &self.base_url,
            "/info/markets/<market>/stats",
            &params,
            &[],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_candles_history(
        &self,
        market_name: &str,
        candle_type: CandleType,
        interval: CandleInterval,
        limit: i64,
        end_time: Option<DateTime<Utc>>,
    ) -> X10Result<WrappedApiResponse<Vec<CandleModel>>> {
        let mut params = HashMap::new();
        params.insert("market", market_name.to_string());
        params.insert("candle_type", candle_type.to_string());
        let url = build_url(
            &self.base_url,
            "/info/candles/<market>/<candle_type>",
            &params,
            &[
                ("interval", QueryValue::Single(interval.to_string())),
                ("limit", QueryValue::Single(limit.to_string())),
                (
                    "endTime",
                    end_time
                        .map(|t| QueryValue::Single(to_epoch_millis(t).to_string()))
                        .unwrap_or(QueryValue::None),
                ),
            ],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_funding_rates_history(
        &self,
        market_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> X10Result<WrappedApiResponse<Vec<FundingRateModel>>> {
        let mut params = HashMap::new();
        params.insert("market", market_name.to_string());
        let url = build_url(
            &self.base_url,
            "/info/<market>/funding",
            &params,
            &[
                (
                    "startTime",
                    QueryValue::Single(to_epoch_millis(start_time).to_string()),
                ),
                (
                    "endTime",
                    QueryValue::Single(to_epoch_millis(end_time).to_string()),
                ),
            ],
        );
        self.http.get(&url, None).await
    }

    pub async fn get_orderbook_snapshot(
        &self,
        market_name: &str,
    ) -> X10Result<WrappedApiResponse<OrderbookUpdateModel>> {
        let mut params = HashMap::new();
        params.insert("market", market_name.to_string());
        let url = build_url(
            &self.base_url,
            "/info/markets/<market>/orderbook",
            &params,
            &[],
        );
        self.http.get(&url, None).await
    }
}

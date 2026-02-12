use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::error::X10Result;
use crate::http::client::HttpClient;
use crate::http::response::WrappedApiResponse;
use crate::types::order::{NewOrderModel, PlacedOrderModel};

use super::url_builder::{build_url, QueryValue};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MassCancelRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    order_ids: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_order_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markets: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cancel_all: Option<bool>,
}

pub struct OrderManagementModule {
    http: Arc<HttpClient>,
    base_url: String,
}

impl OrderManagementModule {
    pub(crate) fn new(http: Arc<HttpClient>, base_url: String) -> Self {
        Self { http, base_url }
    }

    /// Place a new order on the exchange.
    pub async fn place_order(
        &self,
        order: &NewOrderModel,
    ) -> X10Result<WrappedApiResponse<PlacedOrderModel>> {
        let url = build_url(&self.base_url, "/user/order", &HashMap::new(), &[]);
        let body = serde_json::to_value(order)?;
        self.http
            .post::<PlacedOrderModel, Value>(&url, Some(&body), None)
            .await
    }

    /// Cancel an order by its internal ID.
    pub async fn cancel_order(
        &self,
        order_id: i64,
    ) -> X10Result<WrappedApiResponse<Value>> {
        let mut params = HashMap::new();
        params.insert("order_id", order_id.to_string());
        let url = build_url(&self.base_url, "/user/order/<order_id>", &params, &[]);
        self.http.delete::<Value>(&url, None).await
    }

    /// Cancel an order by its external ID.
    pub async fn cancel_order_by_external_id(
        &self,
        external_id: &str,
    ) -> X10Result<WrappedApiResponse<Value>> {
        let url = build_url(
            &self.base_url,
            "/user/order",
            &HashMap::new(),
            &[("externalId", QueryValue::Single(external_id.to_string()))],
        );
        self.http.delete::<Value>(&url, None).await
    }

    /// Mass cancel orders.
    pub async fn mass_cancel(
        &self,
        order_ids: Option<Vec<i64>>,
        external_order_ids: Option<Vec<String>>,
        markets: Option<Vec<String>>,
        cancel_all: Option<bool>,
    ) -> X10Result<WrappedApiResponse<Value>> {
        let url = build_url(
            &self.base_url,
            "/user/order/massCancel",
            &HashMap::new(),
            &[],
        );
        let body = MassCancelRequest {
            order_ids,
            external_order_ids,
            markets,
            cancel_all,
        };
        self.http
            .post::<Value, MassCancelRequest>(&url, Some(&body), None)
            .await
    }
}

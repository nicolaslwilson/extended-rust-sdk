use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

use crate::error::X10Result;
use crate::http::client::HttpClient;
use crate::http::response::WrappedApiResponse;

use super::url_builder::build_url;

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimResponseModel {
    pub id: i64,
}

pub struct TestnetModule {
    http: Arc<HttpClient>,
    base_url: String,
}

impl TestnetModule {
    pub(crate) fn new(http: Arc<HttpClient>, base_url: String) -> Self {
        Self { http, base_url }
    }

    /// Claim testing funds on testnet.
    pub async fn claim_testing_funds(
        &self,
    ) -> X10Result<WrappedApiResponse<ClaimResponseModel>> {
        let url = build_url(&self.base_url, "/user/claim", &HashMap::new(), &[]);
        let body = serde_json::json!({});
        self.http.post(&url, Some(&body), None).await
    }
}

// Note: The Python SDK has retry logic to wait for claim completion.
// Users of this SDK can implement similar retry logic using tokio::time::sleep.

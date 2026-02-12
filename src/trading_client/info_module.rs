use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

use crate::error::X10Result;
use crate::http::client::HttpClient;
use crate::http::response::WrappedApiResponse;

use super::url_builder::build_url;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsModel {
    pub stark_ex_contract_address: String,
}

pub struct InfoModule {
    http: Arc<HttpClient>,
    base_url: String,
}

impl InfoModule {
    pub(crate) fn new(http: Arc<HttpClient>, base_url: String) -> Self {
        Self { http, base_url }
    }

    pub async fn get_settings(&self) -> X10Result<WrappedApiResponse<SettingsModel>> {
        let url = build_url(&self.base_url, "/info/settings", &HashMap::new(), &[]);
        self.http.get(&url, None).await
    }
}

use std::collections::HashMap;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::config::DEFAULT_REQUEST_TIMEOUT_SECS;
use crate::error::{X10Error, X10Result};
use crate::http::response::WrappedApiResponse;
use crate::types::enums::ResponseStatus;

const API_KEY_HEADER: &str = "X-Api-Key";

/// HTTP client wrapping reqwest with auth and error handling.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    api_key: Option<String>,
}

impl HttpClient {
    pub fn new(api_key: Option<String>) -> X10Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(concat!(
                "ExtendedRustTradingClient/",
                env!("CARGO_PKG_VERSION")
            )),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS))
            .build()?;

        Ok(Self { client, api_key })
    }

    fn auth_headers(&self, extra_headers: Option<&HashMap<String, String>>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(ref key) = self.api_key {
            headers.insert(API_KEY_HEADER, HeaderValue::from_str(key).unwrap());
        }
        if let Some(extra) = extra_headers {
            for (k, v) in extra {
                if let (Ok(name), Ok(val)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    HeaderValue::from_str(v),
                ) {
                    headers.insert(name, val);
                }
            }
        }
        headers
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        extra_headers: Option<&HashMap<String, String>>,
    ) -> X10Result<WrappedApiResponse<T>> {
        let headers = self.auth_headers(extra_headers);
        let response = self.client.get(url).headers(headers).send().await?;
        self.handle_response(url, response).await
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: Option<&B>,
        extra_headers: Option<&HashMap<String, String>>,
    ) -> X10Result<WrappedApiResponse<T>> {
        let headers = self.auth_headers(extra_headers);
        let mut req = self.client.post(url).headers(headers);
        if let Some(b) = body {
            req = req.json(b);
        }
        let response = req.send().await?;
        let result: WrappedApiResponse<T> = self.handle_response(url, response).await?;

        if result.status != ResponseStatus::OK || result.error.is_some() {
            let err_msg = result
                .error
                .as_ref()
                .map(|e| format!("code {}: {}", e.code, e.message))
                .unwrap_or_else(|| "unknown error".into());
            return Err(X10Error::Other(format!(
                "Error response from POST {}: {}",
                url, err_msg
            )));
        }

        Ok(result)
    }

    pub async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: Option<&B>,
        extra_headers: Option<&HashMap<String, String>>,
    ) -> X10Result<WrappedApiResponse<T>> {
        let headers = self.auth_headers(extra_headers);
        let mut req = self.client.patch(url).headers(headers);
        if let Some(b) = body {
            req = req.json(b);
        }
        let response = req.send().await?;

        let status = response.status();
        let text = response.text().await?;

        let text = if text.is_empty() {
            tracing::warn!("Empty HTTP {} response from PATCH {}", status, url);
            r#"{"status": "OK"}"#.to_string()
        } else {
            text
        };

        handle_http_errors(url, status.as_u16(), &text)?;
        let result: WrappedApiResponse<T> = serde_json::from_str(&text)?;
        Ok(result)
    }

    pub async fn delete<T: DeserializeOwned>(
        &self,
        url: &str,
        extra_headers: Option<&HashMap<String, String>>,
    ) -> X10Result<WrappedApiResponse<T>> {
        let headers = self.auth_headers(extra_headers);
        let response = self.client.delete(url).headers(headers).send().await?;
        self.handle_response(url, response).await
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        url: &str,
        response: reqwest::Response,
    ) -> X10Result<WrappedApiResponse<T>> {
        let status = response.status();
        let text = response.text().await?;

        handle_http_errors(url, status.as_u16(), &text)?;

        let result: WrappedApiResponse<T> = serde_json::from_str(&text)?;
        Ok(result)
    }
}

fn handle_http_errors(url: &str, status: u16, body: &str) -> X10Result<()> {
    if status == 401 {
        return Err(X10Error::Unauthorized(format!(
            "Unauthorized response from {}: {}",
            url, body
        )));
    }
    if status == 429 {
        return Err(X10Error::RateLimited(format!(
            "Rate limited response from {}: {}",
            url, body
        )));
    }
    if status > 299 {
        return Err(X10Error::Api {
            status,
            body: body.to_string(),
        });
    }
    Ok(())
}

pub mod onboarding;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::error::{X10Error, X10Result};
use crate::http::client::HttpClient;
use crate::http::response::WrappedApiResponse;
use crate::types::account::{AccountModel, ApiKeyRequestModel, ApiKeyResponseModel};
use crate::types::config::EndpointConfig;

use onboarding::{
    get_l2_keys_from_l1_account, get_onboarding_payload, get_sub_account_creation_payload,
    OnboardedClientModel, StarkKeyPair,
};

const L1_AUTH_SIGNATURE_HEADER: &str = "L1_SIGNATURE";
const L1_MESSAGE_TIME_HEADER: &str = "L1_MESSAGE_TIME";
const ACTIVE_ACCOUNT_HEADER: &str = "X-X10-ACTIVE-ACCOUNT";

#[derive(Debug, Clone)]
pub struct OnBoardedAccount {
    pub account: AccountModel,
    pub l2_key_pair: StarkKeyPair,
}

/// User client for onboarding and account management.
///
/// Requires an Ethereum L1 private key for signing onboarding payloads.
pub struct UserClient {
    config: EndpointConfig,
    l1_private_key: Box<dyn Fn() -> String + Send + Sync>,
    http: Arc<HttpClient>,
}

impl UserClient {
    pub fn new(
        endpoint_config: EndpointConfig,
        l1_private_key: impl Fn() -> String + Send + Sync + 'static,
    ) -> X10Result<Self> {
        let http = Arc::new(HttpClient::new(None)?);
        Ok(Self {
            config: endpoint_config,
            l1_private_key: Box::new(l1_private_key),
            http,
        })
    }

    /// Onboard a new account on the exchange.
    pub async fn onboard(
        &self,
        referral_code: Option<&str>,
    ) -> X10Result<OnBoardedAccount> {
        let private_key = (self.l1_private_key)();
        let signing_key = alloy_signer_local::PrivateKeySigner::from_bytes(
            &hex_to_bytes32(&private_key)?,
        )
        .map_err(|e| X10Error::Crypto(format!("Invalid L1 private key: {}", e)))?;

        let key_pair = get_l2_keys_from_l1_account(
            &signing_key,
            0,
            &self.config.signing_domain,
        )
        .await?;

        let payload = get_onboarding_payload(
            &signing_key,
            &self.config.signing_domain,
            &key_pair,
            &self.config.onboarding_url,
            None,
            referral_code.map(|s| s.to_string()),
        )
        .await?;

        let url = format!("{}/auth/onboard", self.config.onboarding_url);
        let body = payload.to_json();
        let response: WrappedApiResponse<OnboardedClientModel> =
            self.http.post(&url, Some(&body), None).await?;

        let onboarded = response
            .data
            .ok_or_else(|| X10Error::Other("No account data returned from onboarding".into()))?;

        Ok(OnBoardedAccount {
            account: onboarded.default_account,
            l2_key_pair: key_pair,
        })
    }

    /// Create a subaccount.
    pub async fn onboard_subaccount(
        &self,
        account_index: i64,
        description: Option<&str>,
    ) -> X10Result<OnBoardedAccount> {
        let private_key = (self.l1_private_key)();
        let signing_key = alloy_signer_local::PrivateKeySigner::from_bytes(
            &hex_to_bytes32(&private_key)?,
        )
        .map_err(|e| X10Error::Crypto(format!("Invalid L1 private key: {}", e)))?;

        let description = description
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Subaccount {}", account_index));

        let address = signing_key.address();
        let now: DateTime<Utc> = Utc::now();
        let auth_time_string = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let request_path = "/auth/onboard/subaccount";
        let l1_message = format!("{}@{}", request_path, auth_time_string);
        let l1_signature = sign_personal_message(&signing_key, l1_message.as_bytes()).await?;

        let key_pair = get_l2_keys_from_l1_account(
            &signing_key,
            account_index,
            &self.config.signing_domain,
        )
        .await?;

        let payload = get_sub_account_creation_payload(
            account_index,
            &format!("{:?}", address),
            &key_pair,
            &description,
            &self.config.onboarding_url,
            None,
        )
        .await?;

        let mut headers = HashMap::new();
        headers.insert(
            L1_AUTH_SIGNATURE_HEADER.to_string(),
            l1_signature,
        );
        headers.insert(L1_MESSAGE_TIME_HEADER.to_string(), auth_time_string);

        let url = format!("{}{}", self.config.onboarding_url, request_path);
        let body = payload.to_json();

        let response: X10Result<WrappedApiResponse<AccountModel>> =
            self.http.post(&url, Some(&body), Some(&headers)).await;

        match response {
            Ok(resp) => {
                let account = resp
                    .data
                    .ok_or_else(|| X10Error::Other("No account data returned".into()))?;
                Ok(OnBoardedAccount {
                    account,
                    l2_key_pair: key_pair,
                })
            }
            Err(X10Error::Api { status: 409, .. }) => {
                // Subaccount already exists, try to find it
                let accounts = self.get_accounts().await?;
                let matching = accounts
                    .into_iter()
                    .find(|a| a.account.account_index == account_index)
                    .ok_or(X10Error::SubAccountExists)?;
                Ok(matching)
            }
            Err(e) => Err(e),
        }
    }

    /// Get all accounts.
    pub async fn get_accounts(&self) -> X10Result<Vec<OnBoardedAccount>> {
        let private_key = (self.l1_private_key)();
        let signing_key = alloy_signer_local::PrivateKeySigner::from_bytes(
            &hex_to_bytes32(&private_key)?,
        )
        .map_err(|e| X10Error::Crypto(format!("Invalid L1 private key: {}", e)))?;

        let request_path = "/api/v1/user/accounts";
        let now: DateTime<Utc> = Utc::now();
        let auth_time_string = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let l1_message = format!("{}@{}", request_path, auth_time_string);
        let l1_signature = sign_personal_message(&signing_key, l1_message.as_bytes()).await?;

        let mut headers = HashMap::new();
        headers.insert(L1_AUTH_SIGNATURE_HEADER.to_string(), l1_signature);
        headers.insert(L1_MESSAGE_TIME_HEADER.to_string(), auth_time_string);

        let url = format!("{}{}", self.config.onboarding_url, request_path);
        let response: WrappedApiResponse<Vec<AccountModel>> =
            self.http.get(&url, Some(&headers)).await?;

        let accounts = response.data.unwrap_or_default();
        let mut result = Vec::new();
        for account in accounts {
            let key_pair = get_l2_keys_from_l1_account(
                &signing_key,
                account.account_index,
                &self.config.signing_domain,
            )
            .await?;
            result.push(OnBoardedAccount {
                account,
                l2_key_pair: key_pair,
            });
        }
        Ok(result)
    }

    /// Create an API key for an account.
    pub async fn create_account_api_key(
        &self,
        account: &AccountModel,
        description: Option<&str>,
    ) -> X10Result<String> {
        let private_key = (self.l1_private_key)();
        let signing_key = alloy_signer_local::PrivateKeySigner::from_bytes(
            &hex_to_bytes32(&private_key)?,
        )
        .map_err(|e| X10Error::Crypto(format!("Invalid L1 private key: {}", e)))?;

        let description = description
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("trading api key for account {}", account.id));

        let request_path = "/api/v1/user/account/api-key";
        let now: DateTime<Utc> = Utc::now();
        let auth_time_string = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let l1_message = format!("{}@{}", request_path, auth_time_string);
        let l1_signature = sign_personal_message(&signing_key, l1_message.as_bytes()).await?;

        let mut headers = HashMap::new();
        headers.insert(L1_AUTH_SIGNATURE_HEADER.to_string(), l1_signature);
        headers.insert(L1_MESSAGE_TIME_HEADER.to_string(), auth_time_string);
        headers.insert(ACTIVE_ACCOUNT_HEADER.to_string(), account.id.to_string());

        let url = format!("{}{}", self.config.onboarding_url, request_path);
        let body = ApiKeyRequestModel { description };

        let response: WrappedApiResponse<ApiKeyResponseModel> =
            self.http.post(&url, Some(&body), Some(&headers)).await?;

        let key_data = response
            .data
            .ok_or_else(|| X10Error::Other("No API key data returned".into()))?;
        Ok(key_data.key)
    }
}

fn hex_to_bytes32(hex_str: &str) -> X10Result<alloy_primitives::B256> {
    let stripped = hex_str.trim_start_matches("0x");
    let bytes = hex::decode(stripped)
        .map_err(|e| X10Error::Crypto(format!("Invalid hex: {}", e)))?;
    if bytes.len() != 32 {
        return Err(X10Error::Crypto(format!(
            "Expected 32 bytes, got {}",
            bytes.len()
        )));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(alloy_primitives::B256::from(arr))
}

async fn sign_personal_message(
    signer: &alloy_signer_local::PrivateKeySigner,
    message: &[u8],
) -> X10Result<String> {
    use alloy_signer::Signer;
    let sig = signer
        .sign_message(message)
        .await
        .map_err(|e| X10Error::Crypto(format!("L1 signing failed: {}", e)))?;
    Ok(format!("0x{}", hex::encode(sig.as_bytes())))
}

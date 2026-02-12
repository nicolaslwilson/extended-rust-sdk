use chrono::{DateTime, Utc};
use serde_json::Value;
use starknet_crypto::Felt;

use crate::error::{X10Error, X10Result};
use crate::types::account::AccountModel;

/// L2 key pair derived from L1 signature.
#[derive(Debug, Clone)]
pub struct StarkKeyPair {
    pub private: Felt,
    pub public: Felt,
}

impl StarkKeyPair {
    pub fn public_hex(&self) -> String {
        format!("{:#x}", self.public)
    }

    pub fn private_hex(&self) -> String {
        format!("{:#x}", self.private)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardedClientModel {
    pub l1_address: String,
    pub default_account: AccountModel,
}

#[derive(Debug, Clone)]
pub struct AccountRegistration {
    pub account_index: i64,
    pub wallet: String,
    pub tos_accepted: bool,
    pub time_string: String,
    pub action: String,
    pub host: String,
}

impl AccountRegistration {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "accountIndex": self.account_index,
            "wallet": self.wallet,
            "tosAccepted": self.tos_accepted,
            "time": self.time_string,
            "action": self.action,
            "host": self.host,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OnboardingPayload {
    pub l1_signature: String,
    pub l2_key: Felt,
    pub l2_r: Felt,
    pub l2_s: Felt,
    pub account_registration: AccountRegistration,
    pub referral_code: Option<String>,
}

impl OnboardingPayload {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "l1Signature": self.l1_signature,
            "l2Key": format!("{:#x}", self.l2_key),
            "l2Signature": {
                "r": format!("{:#x}", self.l2_r),
                "s": format!("{:#x}", self.l2_s),
            },
            "accountCreation": self.account_registration.to_json(),
            "referralCode": self.referral_code,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SubAccountOnboardingPayload {
    pub l2_key: Felt,
    pub l2_r: Felt,
    pub l2_s: Felt,
    pub account_registration: AccountRegistration,
    pub description: String,
}

impl SubAccountOnboardingPayload {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "l2Key": format!("{:#x}", self.l2_key),
            "l2Signature": {
                "r": format!("{:#x}", self.l2_r),
                "s": format!("{:#x}", self.l2_s),
            },
            "accountCreation": self.account_registration.to_json(),
            "description": self.description,
        })
    }
}

/// Derive L2 keys from an L1 account using EIP-712 signing.
pub async fn get_l2_keys_from_l1_account(
    signer: &alloy_signer_local::PrivateKeySigner,
    account_index: i64,
    signing_domain: &str,
) -> X10Result<StarkKeyPair> {
    use alloy_signer::Signer;

    let address = signer.address();

    // Build the struct hash manually since alloy_sol_types requires compile-time types
    // We need to hash: AccountCreation(int8 accountIndex, address wallet, bool tosAccepted)
    let type_hash = alloy_primitives::keccak256(
        b"AccountCreation(int8 accountIndex,address wallet,bool tosAccepted)",
    );

    let mut encoded = Vec::with_capacity(128);
    encoded.extend_from_slice(type_hash.as_slice());
    // int8 accountIndex - left-padded to 32 bytes
    let mut account_index_bytes = [0u8; 32];
    account_index_bytes[31] = account_index as u8;
    encoded.extend_from_slice(&account_index_bytes);
    // address wallet - left-padded to 32 bytes
    let mut address_bytes = [0u8; 32];
    address_bytes[12..32].copy_from_slice(address.as_slice());
    encoded.extend_from_slice(&address_bytes);
    // bool tosAccepted - true = 1
    let mut bool_bytes = [0u8; 32];
    bool_bytes[31] = 1;
    encoded.extend_from_slice(&bool_bytes);

    let struct_hash = alloy_primitives::keccak256(&encoded);

    // Compute domain separator
    let domain_type_hash = alloy_primitives::keccak256(b"EIP712Domain(string name)");
    let name_hash = alloy_primitives::keccak256(signing_domain.as_bytes());
    let mut domain_encoded = Vec::with_capacity(64);
    domain_encoded.extend_from_slice(domain_type_hash.as_slice());
    domain_encoded.extend_from_slice(name_hash.as_slice());
    let domain_separator = alloy_primitives::keccak256(&domain_encoded);

    // Build EIP-712 hash: keccak256("\x19\x01" || domainSeparator || structHash)
    let mut signing_input = Vec::with_capacity(66);
    signing_input.extend_from_slice(&[0x19, 0x01]);
    signing_input.extend_from_slice(domain_separator.as_slice());
    signing_input.extend_from_slice(struct_hash.as_slice());
    let message_hash = alloy_primitives::keccak256(&signing_input);

    // Sign the hash
    let sig = signer
        .sign_hash(&message_hash)
        .await
        .map_err(|e| X10Error::Crypto(format!("EIP-712 signing failed: {}", e)))?;

    let sig_hex = hex::encode(sig.as_bytes());

    // Use rust-crypto-lib-base to derive L2 keypair from the signature
    let private_key = rust_crypto_lib_base::get_private_key_from_eth_signature(&sig_hex)
        .map_err(|e| X10Error::Crypto(format!("L2 key derivation failed: {}", e)))?;

    let public_key = starknet_crypto::get_public_key(&private_key);

    Ok(StarkKeyPair {
        private: private_key,
        public: public_key,
    })
}

/// Create an onboarding payload for initial account registration.
pub async fn get_onboarding_payload(
    signer: &alloy_signer_local::PrivateKeySigner,
    signing_domain: &str,
    key_pair: &StarkKeyPair,
    host: &str,
    time: Option<DateTime<Utc>>,
    referral_code: Option<String>,
) -> X10Result<OnboardingPayload> {
    let address = signer.address();
    let time = time.unwrap_or_else(Utc::now);
    let time_string = time.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let registration = AccountRegistration {
        account_index: 0,
        wallet: format!("{:?}", address),
        tos_accepted: true,
        time_string: time_string.clone(),
        action: "REGISTER".to_string(),
        host: host.to_string(),
    };

    // Sign the registration EIP-712 message
    let l1_signature = sign_registration_eip712(signer, &registration, signing_domain).await?;

    // Create L2 message: pedersen_hash(address_int, public_key)
    let address_bytes = address.as_slice();
    let mut addr_be = [0u8; 32];
    addr_be[12..32].copy_from_slice(address_bytes);
    let address_felt = Felt::from_bytes_be(&addr_be);

    let l2_message = starknet_crypto::pedersen_hash(&address_felt, &key_pair.public);

    let sig = starknet_crypto::sign(&key_pair.private, &l2_message, &Felt::from(1u64))
        .map_err(|e| X10Error::Crypto(format!("L2 signing failed: {}", e)))?;

    Ok(OnboardingPayload {
        l1_signature,
        l2_key: key_pair.public,
        l2_r: sig.r,
        l2_s: sig.s,
        account_registration: registration,
        referral_code,
    })
}

/// Create a subaccount creation payload.
pub async fn get_sub_account_creation_payload(
    account_index: i64,
    l1_address: &str,
    key_pair: &StarkKeyPair,
    description: &str,
    host: &str,
    time: Option<DateTime<Utc>>,
) -> X10Result<SubAccountOnboardingPayload> {
    let time = time.unwrap_or_else(Utc::now);
    let time_string = time.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let registration = AccountRegistration {
        account_index,
        wallet: l1_address.to_string(),
        tos_accepted: true,
        time_string,
        action: "CREATE_SUB_ACCOUNT".to_string(),
        host: host.to_string(),
    };

    // Create L2 message: pedersen_hash(address_int, public_key)
    let address_stripped = l1_address.trim_start_matches("0x");
    let address_bytes = hex::decode(address_stripped)
        .map_err(|e| X10Error::Crypto(format!("Invalid address hex: {}", e)))?;
    let mut addr_be = [0u8; 32];
    let offset = 32 - address_bytes.len();
    addr_be[offset..32].copy_from_slice(&address_bytes);
    let address_felt = Felt::from_bytes_be(&addr_be);

    let l2_message = starknet_crypto::pedersen_hash(&address_felt, &key_pair.public);

    let sig = starknet_crypto::sign(&key_pair.private, &l2_message, &Felt::from(1u64))
        .map_err(|e| X10Error::Crypto(format!("L2 signing failed: {}", e)))?;

    Ok(SubAccountOnboardingPayload {
        l2_key: key_pair.public,
        l2_r: sig.r,
        l2_s: sig.s,
        account_registration: registration,
        description: description.to_string(),
    })
}

async fn sign_registration_eip712(
    signer: &alloy_signer_local::PrivateKeySigner,
    registration: &AccountRegistration,
    signing_domain: &str,
) -> X10Result<String> {
    use alloy_signer::Signer;

    // AccountRegistration EIP-712 type hash
    let type_hash = alloy_primitives::keccak256(
        b"AccountRegistration(int8 accountIndex,address wallet,bool tosAccepted,string time,string action,string host)",
    );

    // Parse the wallet address
    let wallet_addr: alloy_primitives::Address = registration
        .wallet
        .parse()
        .map_err(|e| X10Error::Crypto(format!("Invalid wallet address: {}", e)))?;

    let mut encoded = Vec::with_capacity(256);
    encoded.extend_from_slice(type_hash.as_slice());

    // int8 accountIndex
    let mut account_index_bytes = [0u8; 32];
    account_index_bytes[31] = registration.account_index as u8;
    encoded.extend_from_slice(&account_index_bytes);

    // address wallet
    let mut address_bytes = [0u8; 32];
    address_bytes[12..32].copy_from_slice(wallet_addr.as_slice());
    encoded.extend_from_slice(&address_bytes);

    // bool tosAccepted
    let mut bool_bytes = [0u8; 32];
    bool_bytes[31] = 1;
    encoded.extend_from_slice(&bool_bytes);

    // string time -> keccak256(time)
    let time_hash = alloy_primitives::keccak256(registration.time_string.as_bytes());
    encoded.extend_from_slice(time_hash.as_slice());

    // string action -> keccak256(action)
    let action_hash = alloy_primitives::keccak256(registration.action.as_bytes());
    encoded.extend_from_slice(action_hash.as_slice());

    // string host -> keccak256(host)
    let host_hash = alloy_primitives::keccak256(registration.host.as_bytes());
    encoded.extend_from_slice(host_hash.as_slice());

    let struct_hash = alloy_primitives::keccak256(&encoded);

    // Domain separator
    let domain_type_hash = alloy_primitives::keccak256(b"EIP712Domain(string name)");
    let name_hash = alloy_primitives::keccak256(signing_domain.as_bytes());
    let mut domain_encoded = Vec::with_capacity(64);
    domain_encoded.extend_from_slice(domain_type_hash.as_slice());
    domain_encoded.extend_from_slice(name_hash.as_slice());
    let domain_separator = alloy_primitives::keccak256(&domain_encoded);

    // EIP-712 hash
    let mut signing_input = Vec::with_capacity(66);
    signing_input.extend_from_slice(&[0x19, 0x01]);
    signing_input.extend_from_slice(domain_separator.as_slice());
    signing_input.extend_from_slice(struct_hash.as_slice());
    let message_hash = alloy_primitives::keccak256(&signing_input);

    let sig = signer
        .sign_hash(&message_hash)
        .await
        .map_err(|e| X10Error::Crypto(format!("EIP-712 signing failed: {}", e)))?;

    Ok(format!("0x{}", hex::encode(sig.as_bytes())))
}

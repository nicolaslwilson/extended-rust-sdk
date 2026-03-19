use chrono::Duration;
use rust_decimal::Decimal;

use crate::types::account::StarkPerpetualAccount;
use crate::types::common::{HexValue, SettlementSignatureModel};
use crate::types::config::EndpointConfig;
use crate::types::withdrawal::{StarkWithdrawalSettlement, Timestamp, WithdrawalRequest};
use crate::utils::nonce::generate_nonce;
use crate::utils::time::utc_now;

fn calc_expiration_timestamp() -> i64 {
    let expire_time = utc_now();
    let expire_with_buffer = expire_time + Duration::days(15);
    let secs = expire_with_buffer.timestamp();
    let nanos = expire_with_buffer.timestamp_subsec_nanos();
    if nanos > 0 {
        secs + 1
    } else {
        secs
    }
}

pub fn create_withdrawal_object(
    amount: Decimal,
    recipient_stark_address: &str,
    stark_account: &StarkPerpetualAccount,
    config: &EndpointConfig,
    account_id: i64,
    chain_id: &str,
    description: Option<String>,
    nonce: Option<u32>,
    quote_id: Option<String>,
) -> Result<WithdrawalRequest, String> {
    let expiration_timestamp = calc_expiration_timestamp();

    let scale = Decimal::from(10u64.pow(config.collateral_decimals));
    let scaled = amount * scale;
    let stark_amount = scaled
        .round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointNearestEven)
        .to_string()
        .parse::<u64>()
        .map_err(|e| format!("amount conversion error: {}", e))?;

    let domain = &config.starknet_domain;
    let nonce = nonce.unwrap_or_else(generate_nonce);

    let collateral_id_hex = &config.collateral_asset_on_chain_id;

    let hash = rust_crypto_lib_base::get_withdrawal_hash(
        recipient_stark_address.to_string(),
        stark_account.vault().to_string(),
        collateral_id_hex.clone(),
        stark_amount.to_string(),
        expiration_timestamp.to_string(),
        nonce.to_string(),
        format!("{:#066x}", stark_account.public_key()),
        domain.name.clone(),
        domain.version.clone(),
        domain.chain_id.clone(),
        domain.revision.clone(),
    )?;

    let (sig_r, sig_s) = stark_account.sign(hash);

    let recipient_hex = recipient_stark_address.trim();
    let recipient_hex = if recipient_hex.starts_with("0x") || recipient_hex.starts_with("0X") {
        recipient_hex.to_string()
    } else {
        format!("0x{}", recipient_hex)
    };

    let collateral_id_int = i128::from_str_radix(
        collateral_id_hex.trim_start_matches("0x"),
        16,
    )
    .map_err(|e| format!("invalid collateral_asset_on_chain_id: {}", e))?;

    let settlement = StarkWithdrawalSettlement {
        recipient: HexValue::from_hex_string(recipient_hex),
        position_id: stark_account.vault(),
        collateral_id: HexValue::new(collateral_id_int),
        amount: stark_amount as i64,
        expiration: Timestamp {
            seconds: expiration_timestamp,
        },
        salt: nonce,
        signature: SettlementSignatureModel {
            r: HexValue::from_hex_string(format!("{:#x}", sig_r)),
            s: HexValue::from_hex_string(format!("{:#x}", sig_s)),
        },
    };

    Ok(WithdrawalRequest {
        account_id,
        amount,
        description,
        settlement,
        chain_id: chain_id.to_string(),
        quote_id,
        asset: "USD".to_string(),
    })
}

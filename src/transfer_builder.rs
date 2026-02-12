use chrono::Duration;
use rust_decimal::Decimal;
use starknet_crypto::Felt;

use crate::types::account::StarkPerpetualAccount;
use crate::types::common::{HexValue, SettlementSignatureModel};
use crate::types::config::EndpointConfig;
use crate::types::transfer::{OnChainPerpetualTransferModel, StarkTransferSettlement};
use crate::utils::nonce::generate_nonce;
use crate::utils::time::utc_now;

fn calc_expiration_timestamp() -> i64 {
    let expire_time = utc_now() + Duration::days(7);
    let expire_with_buffer = expire_time + Duration::days(14);
    let secs = expire_with_buffer.timestamp();
    let nanos = expire_with_buffer.timestamp_subsec_nanos();
    if nanos > 0 {
        secs + 1
    } else {
        secs
    }
}

pub fn create_transfer_object(
    from_vault: u64,
    to_vault: u64,
    to_l2_key: Felt,
    amount: Decimal,
    config: &EndpointConfig,
    stark_account: &StarkPerpetualAccount,
    nonce: Option<u32>,
) -> Result<OnChainPerpetualTransferModel, String> {
    let expiration_timestamp = calc_expiration_timestamp();

    // Scale amount by collateral decimals
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

    let hash = rust_crypto_lib_base::get_transfer_hash(
        to_vault.to_string(),
        from_vault.to_string(),
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

    let collateral_id_int = i128::from_str_radix(
        collateral_id_hex.trim_start_matches("0x"),
        16,
    )
    .map_err(|e| format!("invalid collateral_asset_on_chain_id: {}", e))?;

    let settlement = StarkTransferSettlement {
        amount: stark_amount as i64,
        asset_id: HexValue::new(collateral_id_int),
        expiration_timestamp,
        nonce,
        receiver_position_id: to_vault,
        receiver_public_key: HexValue::from_hex_string(format!("{:#x}", to_l2_key)),
        sender_position_id: from_vault,
        sender_public_key: HexValue::from_hex_string(format!("{:#x}", stark_account.public_key())),
        signature: SettlementSignatureModel {
            r: HexValue::from_hex_string(format!("{:#x}", sig_r)),
            s: HexValue::from_hex_string(format!("{:#x}", sig_s)),
        },
    };

    Ok(OnChainPerpetualTransferModel {
        from_vault,
        to_vault,
        amount,
        settlement,
        transferred_asset: config.collateral_asset_id.clone(),
    })
}

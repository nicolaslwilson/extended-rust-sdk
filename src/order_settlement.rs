use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use starknet_crypto::Felt;

use crate::amounts::{HumanReadableAmount, StarkAmount, ROUNDING_BUY, ROUNDING_FEE, ROUNDING_SELL};
use crate::types::common::{HexValue, SettlementSignatureModel};
use crate::types::config::StarknetDomain;
use crate::types::enums::OrderSide;
use crate::types::fee::TradingFeeModel;
use crate::types::market::MarketModel;
use crate::types::order::{StarkDebuggingOrderAmountsModel, StarkSettlementModel};

/// Result of creating settlement data for an order.
pub struct OrderSettlementData {
    pub synthetic_amount_human: HumanReadableAmount,
    pub order_hash: Felt,
    pub settlement: StarkSettlementModel,
    pub debugging_amounts: StarkDebuggingOrderAmountsModel,
}

/// Context for creating order settlement data.
pub struct SettlementDataCtx<'a> {
    pub market: &'a MarketModel,
    pub fees: &'a TradingFeeModel,
    pub builder_fee: Option<Decimal>,
    pub nonce: u32,
    pub collateral_position_id: u64,
    pub expire_time: DateTime<Utc>,
    pub signer: &'a dyn Fn(Felt) -> (Felt, Felt),
    pub public_key: Felt,
    pub starknet_domain: &'a StarknetDomain,
}

fn calc_settlement_expiration(expiration_timestamp: DateTime<Utc>) -> i64 {
    let expire_with_buffer = expiration_timestamp + Duration::days(14);
    // Ceiling of seconds
    let secs = expire_with_buffer.timestamp();
    let nanos = expire_with_buffer.timestamp_subsec_nanos();
    if nanos > 0 {
        secs + 1
    } else {
        secs
    }
}

pub fn hash_order(
    amount_synthetic: &StarkAmount,
    amount_collateral: &StarkAmount,
    max_fee: &StarkAmount,
    nonce: u32,
    position_id: u64,
    expiration_timestamp: DateTime<Utc>,
    public_key: Felt,
    starknet_domain: &StarknetDomain,
) -> Result<Felt, String> {
    let synthetic_asset = &amount_synthetic.asset;
    let collateral_asset = &amount_collateral.asset;

    let base_asset_id_hex = &synthetic_asset.settlement_external_id;
    let quote_asset_id_hex = &collateral_asset.settlement_external_id;
    let fee_asset_id_hex = &collateral_asset.settlement_external_id;

    let expiration = calc_settlement_expiration(expiration_timestamp);

    let hash = rust_crypto_lib_base::get_order_hash(
        position_id.to_string(),
        base_asset_id_hex.clone(),
        amount_synthetic.value.to_string(),
        quote_asset_id_hex.clone(),
        amount_collateral.value.to_string(),
        fee_asset_id_hex.clone(),
        max_fee.value.to_string(),
        expiration.to_string(),
        nonce.to_string(),
        format!("{:#066x}", public_key),
        starknet_domain.name.clone(),
        starknet_domain.version.clone(),
        starknet_domain.chain_id.clone(),
        starknet_domain.revision.clone(),
    )?;

    Ok(hash)
}

/// Create order settlement data (settlement signature, amounts, hash).
pub fn create_order_settlement_data(
    side: OrderSide,
    synthetic_amount: Decimal,
    price: Decimal,
    ctx: &SettlementDataCtx,
) -> Result<OrderSettlementData, String> {
    let is_buying = side == OrderSide::BUY;
    let rounding = if is_buying { ROUNDING_BUY } else { ROUNDING_SELL };

    let synthetic_amount_human =
        HumanReadableAmount::new(synthetic_amount, ctx.market.synthetic_asset());
    let collateral_amount_human =
        HumanReadableAmount::new(synthetic_amount * price, ctx.market.collateral_asset());

    let total_fee = ctx.fees.taker_fee_rate + ctx.builder_fee.unwrap_or(Decimal::ZERO);
    let fee_amount_human =
        HumanReadableAmount::new(total_fee * collateral_amount_human.value, ctx.market.collateral_asset());

    let stark_collateral = collateral_amount_human.to_stark_amount(rounding);
    let stark_synthetic = synthetic_amount_human.to_stark_amount(rounding);
    let stark_fee = fee_amount_human.to_stark_amount(ROUNDING_FEE);

    let stark_collateral = if is_buying {
        stark_collateral.negate()
    } else {
        stark_collateral
    };
    let stark_synthetic = if is_buying {
        stark_synthetic
    } else {
        stark_synthetic.negate()
    };

    let debugging_amounts = StarkDebuggingOrderAmountsModel {
        collateral_amount: Decimal::from(stark_collateral.value),
        fee_amount: Decimal::from(stark_fee.value),
        synthetic_amount: Decimal::from(stark_synthetic.value),
    };

    let order_hash = hash_order(
        &stark_synthetic,
        &stark_collateral,
        &stark_fee,
        ctx.nonce,
        ctx.collateral_position_id,
        ctx.expire_time,
        ctx.public_key,
        ctx.starknet_domain,
    )?;

    let (sig_r, sig_s) = (ctx.signer)(order_hash);
    let settlement = StarkSettlementModel {
        signature: SettlementSignatureModel {
            r: HexValue::from_hex_string(format!("{:#x}", sig_r)),
            s: HexValue::from_hex_string(format!("{:#x}", sig_s)),
        },
        stark_key: HexValue::from_hex_string(format!("{:#x}", ctx.public_key)),
        collateral_position: Decimal::from(ctx.collateral_position_id),
    };

    Ok(OrderSettlementData {
        synthetic_amount_human,
        order_hash,
        settlement,
        debugging_amounts,
    })
}

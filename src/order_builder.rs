use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use starknet_crypto::Felt;

use crate::error::{X10Error, X10Result};
use crate::order_settlement::{create_order_settlement_data, SettlementDataCtx};
use crate::types::account::StarkPerpetualAccount;
use crate::types::config::StarknetDomain;
use crate::types::enums::*;
use crate::types::fee::DEFAULT_FEES;
use crate::types::market::MarketModel;
use crate::types::order::{CreateOrderTpslTriggerModel, NewOrderModel};
use crate::utils::nonce::generate_nonce;
use crate::utils::time::{to_epoch_millis, utc_now};

/// Parameters for a take-profit or stop-loss trigger.
#[derive(Debug, Clone)]
pub struct OrderTpslTriggerParam {
    pub trigger_price: Decimal,
    pub trigger_price_type: OrderTriggerPriceType,
    pub price: Decimal,
    pub price_type: OrderPriceType,
}

fn get_opposite_side(side: OrderSide) -> OrderSide {
    match side {
        OrderSide::BUY => OrderSide::SELL,
        OrderSide::SELL => OrderSide::BUY,
    }
}

/// Create an order object matching the Python SDK's `create_order_object`.
#[allow(clippy::too_many_arguments)]
pub fn create_order_object(
    account: &StarkPerpetualAccount,
    market: &MarketModel,
    amount_of_synthetic: Decimal,
    price: Decimal,
    side: OrderSide,
    starknet_domain: &StarknetDomain,
    post_only: bool,
    previous_order_external_id: Option<String>,
    expire_time: Option<DateTime<Utc>>,
    order_external_id: Option<String>,
    time_in_force: TimeInForce,
    self_trade_protection_level: SelfTradeProtectionLevel,
    nonce: Option<u32>,
    builder_fee: Option<Decimal>,
    builder_id: Option<i64>,
    reduce_only: bool,
    tp_sl_type: Option<OrderTpslType>,
    take_profit: Option<OrderTpslTriggerParam>,
    stop_loss: Option<OrderTpslTriggerParam>,
) -> X10Result<NewOrderModel> {
    if time_in_force == TimeInForce::FOK {
        return Err(X10Error::Validation(
            "FOK time in force is not supported".into(),
        ));
    }

    let expire_time = expire_time.unwrap_or_else(|| utc_now() + Duration::hours(1));

    if tp_sl_type == Some(OrderTpslType::POSITION) {
        return Err(X10Error::Validation(
            "POSITION TPSL type is not supported yet".into(),
        ));
    }

    if let Some(ref tp) = take_profit {
        if tp.price_type == OrderPriceType::MARKET {
            return Err(X10Error::Validation(
                "TPSL MARKET price type is not supported yet".into(),
            ));
        }
    }
    if let Some(ref sl) = stop_loss {
        if sl.price_type == OrderPriceType::MARKET {
            return Err(X10Error::Validation(
                "TPSL MARKET price type is not supported yet".into(),
            ));
        }
    }

    let nonce = nonce.unwrap_or_else(generate_nonce);

    let fees = account
        .trading_fee()
        .get(&market.name)
        .cloned()
        .unwrap_or_else(|| DEFAULT_FEES.clone());

    let signer = |hash: Felt| -> (Felt, Felt) { account.sign(hash) };

    let ctx = SettlementDataCtx {
        market,
        fees: &fees,
        builder_fee,
        nonce,
        collateral_position_id: account.vault(),
        expire_time,
        signer: &signer,
        public_key: account.public_key(),
        starknet_domain,
    };

    let settlement_data =
        create_order_settlement_data(side, amount_of_synthetic, price, &ctx)
            .map_err(X10Error::Crypto)?;

    // Build TPSL triggers
    let tp_trigger_model = if let Some(ref tp) = take_profit {
        let tp_settlement = create_order_settlement_data(
            get_opposite_side(side),
            amount_of_synthetic,
            tp.price,
            &ctx,
        )
        .map_err(X10Error::Crypto)?;
        Some(CreateOrderTpslTriggerModel {
            trigger_price: tp.trigger_price,
            trigger_price_type: tp.trigger_price_type,
            price: tp.price,
            price_type: tp.price_type,
            settlement: tp_settlement.settlement,
            debugging_amounts: Some(tp_settlement.debugging_amounts),
        })
    } else {
        None
    };

    let sl_trigger_model = if let Some(ref sl) = stop_loss {
        let sl_settlement = create_order_settlement_data(
            get_opposite_side(side),
            amount_of_synthetic,
            sl.price,
            &ctx,
        )
        .map_err(X10Error::Crypto)?;
        Some(CreateOrderTpslTriggerModel {
            trigger_price: sl.trigger_price,
            trigger_price_type: sl.trigger_price_type,
            price: sl.price,
            price_type: sl.price_type,
            settlement: sl_settlement.settlement,
            debugging_amounts: Some(sl_settlement.debugging_amounts),
        })
    } else {
        None
    };

    let order_id = order_external_id
        .unwrap_or_else(|| format!("{}", settlement_data.order_hash));

    let order = NewOrderModel {
        id: order_id,
        market: market.name.clone(),
        order_type: OrderType::LIMIT,
        side,
        qty: settlement_data.synthetic_amount_human.value,
        price,
        post_only,
        reduce_only,
        time_in_force,
        expiry_epoch_millis: to_epoch_millis(expire_time),
        fee: fees.taker_fee_rate,
        nonce: Decimal::from(nonce),
        self_trade_protection_level,
        cancel_id: previous_order_external_id,
        settlement: Some(settlement_data.settlement),
        trigger: None,
        tp_sl_type,
        take_profit: tp_trigger_model,
        stop_loss: sl_trigger_model,
        debugging_amounts: Some(settlement_data.debugging_amounts),
        builder_fee,
        builder_id,
    };

    Ok(order)
}

/// Builder pattern for constructing orders.
pub struct OrderBuilder<'a> {
    account: &'a StarkPerpetualAccount,
    market: &'a MarketModel,
    side: OrderSide,
    amount: Decimal,
    price: Decimal,
    domain: &'a StarknetDomain,
    post_only: bool,
    reduce_only: bool,
    previous_order_external_id: Option<String>,
    expire_time: Option<DateTime<Utc>>,
    order_external_id: Option<String>,
    time_in_force: TimeInForce,
    self_trade_protection_level: SelfTradeProtectionLevel,
    nonce: Option<u32>,
    builder_fee: Option<Decimal>,
    builder_id: Option<i64>,
    tp_sl_type: Option<OrderTpslType>,
    take_profit: Option<OrderTpslTriggerParam>,
    stop_loss: Option<OrderTpslTriggerParam>,
}

impl<'a> OrderBuilder<'a> {
    pub fn new(
        account: &'a StarkPerpetualAccount,
        market: &'a MarketModel,
        side: OrderSide,
        amount: Decimal,
        price: Decimal,
        domain: &'a StarknetDomain,
    ) -> Self {
        Self {
            account,
            market,
            side,
            amount,
            price,
            domain,
            post_only: false,
            reduce_only: false,
            previous_order_external_id: None,
            expire_time: None,
            order_external_id: None,
            time_in_force: TimeInForce::GTT,
            self_trade_protection_level: SelfTradeProtectionLevel::ACCOUNT,
            nonce: None,
            builder_fee: None,
            builder_id: None,
            tp_sl_type: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    pub fn post_only(mut self, v: bool) -> Self {
        self.post_only = v;
        self
    }

    pub fn reduce_only(mut self, v: bool) -> Self {
        self.reduce_only = v;
        self
    }

    pub fn time_in_force(mut self, v: TimeInForce) -> Self {
        self.time_in_force = v;
        self
    }

    pub fn expire_time(mut self, v: DateTime<Utc>) -> Self {
        self.expire_time = Some(v);
        self
    }

    pub fn nonce(mut self, v: u32) -> Self {
        self.nonce = Some(v);
        self
    }

    pub fn external_id(mut self, v: String) -> Self {
        self.order_external_id = Some(v);
        self
    }

    pub fn cancel_id(mut self, v: String) -> Self {
        self.previous_order_external_id = Some(v);
        self
    }

    pub fn self_trade_protection(mut self, v: SelfTradeProtectionLevel) -> Self {
        self.self_trade_protection_level = v;
        self
    }

    pub fn builder_fee(mut self, fee: Decimal, id: i64) -> Self {
        self.builder_fee = Some(fee);
        self.builder_id = Some(id);
        self
    }

    pub fn tp_sl_type(mut self, v: OrderTpslType) -> Self {
        self.tp_sl_type = Some(v);
        self
    }

    pub fn take_profit(mut self, v: OrderTpslTriggerParam) -> Self {
        self.take_profit = Some(v);
        self
    }

    pub fn stop_loss(mut self, v: OrderTpslTriggerParam) -> Self {
        self.stop_loss = Some(v);
        self
    }

    pub fn build(self) -> X10Result<NewOrderModel> {
        create_order_object(
            self.account,
            self.market,
            self.amount,
            self.price,
            self.side,
            self.domain,
            self.post_only,
            self.previous_order_external_id,
            self.expire_time,
            self.order_external_id,
            self.time_in_force,
            self.self_trade_protection_level,
            self.nonce,
            self.builder_fee,
            self.builder_id,
            self.reduce_only,
            self.tp_sl_type,
            self.take_profit,
            self.stop_loss,
        )
    }
}

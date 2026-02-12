use std::str::FromStr;

use chrono::{Duration, NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;

use extended_rust_sdk::types::account::StarkPerpetualAccount;
use extended_rust_sdk::types::config::TESTNET_CONFIG;
use extended_rust_sdk::types::enums::*;
use extended_rust_sdk::types::market::MarketModel;
use extended_rust_sdk::http::response::WrappedApiResponse;
use extended_rust_sdk::order_builder::{create_order_object, OrderTpslTriggerParam};

const FROZEN_NONCE: u32 = 1473459052;

fn create_trading_account() -> StarkPerpetualAccount {
    StarkPerpetualAccount::new(
        10002,
        "0x7a7ff6fd3cab02ccdcd4a572563f5976f8976899b03a39773795a3c486d4986",
        "0x61c5e7e8339b7d56f197f54ea91b776776690e3232313de0f2ecbd0ef76f466",
        "dummy_api_key",
    )
}

fn get_btc_usd_market_json() -> &'static str {
    r#"
    {
        "status": "OK",
        "data": [
            {
                "name": "BTC-USD",
                "category": "L1",
                "assetName": "BTC",
                "assetPrecision": 5,
                "collateralAssetName": "USD",
                "collateralAssetPrecision": 6,
                "active": true,
                "marketStats": {
                    "dailyVolume": "2410800.768021",
                    "dailyVolumeBase": "37.94502",
                    "dailyPriceChange": "969.9",
                    "dailyPriceChangePercentage": "0.02",
                    "dailyLow": "62614.8",
                    "dailyHigh": "64421.1",
                    "lastPrice": "64280.0",
                    "askPrice": "64268.2",
                    "bidPrice": "64235.9",
                    "markPrice": "64267.380482593245",
                    "indexPrice": "64286.409493065992",
                    "fundingRate": "-0.000034",
                    "nextFundingRate": 1715072400000,
                    "openInterest": "150629.886375",
                    "openInterestBase": "2.34380"
                },
                "tradingConfig": {
                    "minOrderSize": "0.0001",
                    "minOrderSizeChange": "0.00001",
                    "minPriceChange": "0.1",
                    "maxMarketOrderValue": "1000000",
                    "maxLimitOrderValue": "5000000",
                    "maxPositionValue": "10000000",
                    "maxLeverage": "50.00",
                    "maxNumOrders": "200",
                    "limitPriceCap": "0.05",
                    "limitPriceFloor": "0.05",
                    "riskFactorConfig": [
                        {"upperBound": "400000", "riskFactor": "0.02"},
                        {"upperBound": "1000000000", "riskFactor": "1"}
                    ]
                },
                "l2Config": {
                    "type": "STARKX",
                    "collateralId": "0x31857064564ed0ff978e687456963cba09c2c6985d8f9300a1de4962fafa054",
                    "syntheticId": "0x4254432d3600000000000000000000",
                    "syntheticResolution": 1000000,
                    "collateralResolution": 1000000
                }
            }
        ]
    }
    "#
}

fn create_btc_usd_market() -> MarketModel {
    let response: WrappedApiResponse<Vec<MarketModel>> =
        serde_json::from_str(get_btc_usd_market_json()).expect("Failed to parse market JSON");
    response.data.unwrap().into_iter().next().unwrap()
}

/// Port of Python test: test_create_sell_order_with_default_expiration
///
/// Python uses freeze_time("2024-01-05 01:08:56.860694") then moves to "2024-01-05 01:08:57".
/// Default expire = utc_now() + 1h = 2024-01-05T02:08:57Z
#[test]
fn test_create_sell_order_with_default_expiration() {
    let account = create_trading_account();
    let market = create_btc_usd_market();

    // Equivalent of freeze_time move_to("2024-01-05 01:08:57") + 1h default
    let expire_time = Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(2024, 1, 5)
            .unwrap()
            .and_hms_opt(2, 8, 57)
            .unwrap(),
    );

    let order = create_order_object(
        &account,
        &market,
        Decimal::from_str("0.00100000").unwrap(),
        Decimal::from_str("43445.11680000").unwrap(),
        OrderSide::SELL,
        &TESTNET_CONFIG.starknet_domain,
        false,                                    // post_only
        None,                                     // previous_order_external_id
        Some(expire_time),                        // expire_time
        None,                                     // order_external_id
        TimeInForce::GTT,
        SelfTradeProtectionLevel::ACCOUNT,
        Some(FROZEN_NONCE),                       // nonce
        None,                                     // builder_fee
        None,                                     // builder_id
        false,                                    // reduce_only
        None,                                     // tp_sl_type
        None,                                     // take_profit
        None,                                     // stop_loss
    )
    .expect("create_order_object failed");

    // Verify the order ID (decimal representation of the order hash)
    assert_eq!(
        order.id,
        "529621978301228831750156704671293558063128025271079340676658105549022202327"
    );

    // Verify expiry
    assert_eq!(order.expiry_epoch_millis, 1704420537000);

    // Verify settlement signature
    let settlement = order.settlement.as_ref().unwrap();
    assert_eq!(
        settlement.signature.r.to_hex_string(),
        "0x3d17d8b9652e5f60d40d079653cfa92b1065ea8cf159609a3c390070dcd44f7"
    );
    assert_eq!(
        settlement.signature.s.to_hex_string(),
        "0x76a6deccbc84ac324f695cfbde80e0ed62443e95f5dcd8722d12650ccc122e5"
    );
    assert_eq!(
        settlement.stark_key.to_hex_string(),
        "0x61c5e7e8339b7d56f197f54ea91b776776690e3232313de0f2ecbd0ef76f466"
    );

    // Verify debugging amounts
    let amounts = order.debugging_amounts.as_ref().unwrap();
    assert_eq!(amounts.collateral_amount, Decimal::from(43445116));
    assert_eq!(amounts.fee_amount, Decimal::from(21723));
    assert_eq!(amounts.synthetic_amount, Decimal::from(-1000));

    // Verify other fields
    assert_eq!(order.market, "BTC-USD");
    assert_eq!(order.side, OrderSide::SELL);
    assert_eq!(order.order_type, OrderType::LIMIT);
    assert!(!order.post_only);
    assert!(!order.reduce_only);
    assert_eq!(order.time_in_force, TimeInForce::GTT);
    assert_eq!(order.self_trade_protection_level, SelfTradeProtectionLevel::ACCOUNT);
    assert_eq!(order.nonce, Decimal::from(FROZEN_NONCE));
    assert!(order.cancel_id.is_none());
    assert!(order.trigger.is_none());
    assert!(order.tp_sl_type.is_none());
    assert!(order.take_profit.is_none());
    assert!(order.stop_loss.is_none());
    assert!(order.builder_fee.is_none());
    assert!(order.builder_id.is_none());
}

/// Port of Python test: test_create_sell_order
///
/// Python uses freeze_time("2024-01-05 01:08:56.860694").
/// expire_time = utc_now() + 14 days = 2024-01-19T01:08:56.860694Z
#[test]
fn test_create_sell_order() {
    let account = create_trading_account();
    let market = create_btc_usd_market();

    let frozen_time = Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(2024, 1, 5)
            .unwrap()
            .and_hms_micro_opt(1, 8, 56, 860694)
            .unwrap(),
    );
    let expire_time = frozen_time + Duration::days(14);

    let order = create_order_object(
        &account,
        &market,
        Decimal::from_str("0.00100000").unwrap(),
        Decimal::from_str("43445.11680000").unwrap(),
        OrderSide::SELL,
        &TESTNET_CONFIG.starknet_domain,
        false,
        None,
        Some(expire_time),
        None,
        TimeInForce::GTT,
        SelfTradeProtectionLevel::ACCOUNT,
        Some(FROZEN_NONCE),
        None,
        None,
        false,
        None,
        None,
        None,
    )
    .expect("create_order_object failed");

    assert_eq!(
        order.id,
        "2969335148777495210033041829700798003994871688044444919524700744667647811801"
    );
    assert_eq!(order.expiry_epoch_millis, 1705626536861);

    let settlement = order.settlement.as_ref().unwrap();
    assert_eq!(
        settlement.signature.r.to_hex_string(),
        "0x604ef07147d4251385eaaa630e6a71db8f0a8c7cb33021c98698047db80edfa"
    );
    assert_eq!(
        settlement.signature.s.to_hex_string(),
        "0x6c707d9a06604d3f8ffd34378bf4fce7c0aaf50cba4cf37c3525c323106cda5"
    );

    let amounts = order.debugging_amounts.as_ref().unwrap();
    assert_eq!(amounts.collateral_amount, Decimal::from(43445116));
    assert_eq!(amounts.fee_amount, Decimal::from(21723));
    assert_eq!(amounts.synthetic_amount, Decimal::from(-1000));
}

/// Port of Python test: test_create_buy_order
///
/// Same frozen time, but BUY side with CLIENT self-trade protection.
#[test]
fn test_create_buy_order() {
    let account = create_trading_account();
    let market = create_btc_usd_market();

    let frozen_time = Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(2024, 1, 5)
            .unwrap()
            .and_hms_micro_opt(1, 8, 56, 860694)
            .unwrap(),
    );
    let expire_time = frozen_time + Duration::days(14);

    let order = create_order_object(
        &account,
        &market,
        Decimal::from_str("0.00100000").unwrap(),
        Decimal::from_str("43445.11680000").unwrap(),
        OrderSide::BUY,
        &TESTNET_CONFIG.starknet_domain,
        false,
        None,
        Some(expire_time),
        None,
        TimeInForce::GTT,
        SelfTradeProtectionLevel::CLIENT,
        Some(FROZEN_NONCE),
        None,
        None,
        false,
        None,
        None,
        None,
    )
    .expect("create_order_object failed");

    assert_eq!(
        order.id,
        "2495374044666992118771096772295242242651427695217815113349321039194683172848"
    );
    assert_eq!(order.expiry_epoch_millis, 1705626536861);
    assert_eq!(order.self_trade_protection_level, SelfTradeProtectionLevel::CLIENT);

    let settlement = order.settlement.as_ref().unwrap();
    assert_eq!(
        settlement.signature.r.to_hex_string(),
        "0xa55625c7d5f1b85bed22556fc805224b8363074979cf918091d9ddb1403e13"
    );
    assert_eq!(
        settlement.signature.s.to_hex_string(),
        "0x504caf634d859e643569743642ccf244434322859b2421d76f853af43ae7a46"
    );

    // BUY: collateral is negative (outgoing), synthetic is positive (incoming)
    let amounts = order.debugging_amounts.as_ref().unwrap();
    assert_eq!(amounts.collateral_amount, Decimal::from(-43445117));
    assert_eq!(amounts.fee_amount, Decimal::from(21723));
    assert_eq!(amounts.synthetic_amount, Decimal::from(1000));
}

/// Port of Python test: test_create_buy_order_with_tpsl
///
/// BUY order with take-profit and stop-loss triggers.
#[test]
fn test_create_buy_order_with_tpsl() {
    let account = create_trading_account();
    let market = create_btc_usd_market();

    let frozen_time = Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(2024, 1, 5)
            .unwrap()
            .and_hms_micro_opt(1, 8, 56, 860694)
            .unwrap(),
    );
    let expire_time = frozen_time + Duration::days(14);

    let order = create_order_object(
        &account,
        &market,
        Decimal::from_str("0.00100000").unwrap(),
        Decimal::from_str("43445.11680000").unwrap(),
        OrderSide::BUY,
        &TESTNET_CONFIG.starknet_domain,
        false,
        None,
        Some(expire_time),
        None,
        TimeInForce::GTT,
        SelfTradeProtectionLevel::CLIENT,
        Some(FROZEN_NONCE),
        None,
        None,
        false,
        None,
        Some(OrderTpslTriggerParam {
            trigger_price: Decimal::from(49000),
            trigger_price_type: OrderTriggerPriceType::MARK,
            price: Decimal::from(50000),
            price_type: OrderPriceType::LIMIT,
        }),
        Some(OrderTpslTriggerParam {
            trigger_price: Decimal::from(40000),
            trigger_price_type: OrderTriggerPriceType::MARK,
            price: Decimal::from(39000),
            price_type: OrderPriceType::LIMIT,
        }),
    )
    .expect("create_order_object failed");

    // Main order has same settlement as test_create_buy_order (same inputs)
    assert_eq!(
        order.id,
        "2495374044666992118771096772295242242651427695217815113349321039194683172848"
    );

    let settlement = order.settlement.as_ref().unwrap();
    assert_eq!(
        settlement.signature.r.to_hex_string(),
        "0xa55625c7d5f1b85bed22556fc805224b8363074979cf918091d9ddb1403e13"
    );
    assert_eq!(
        settlement.signature.s.to_hex_string(),
        "0x504caf634d859e643569743642ccf244434322859b2421d76f853af43ae7a46"
    );

    // Take-profit trigger (opposite side = SELL, price = 50000)
    let tp = order.take_profit.as_ref().expect("expected take_profit");
    assert_eq!(tp.trigger_price, Decimal::from(49000));
    assert_eq!(tp.trigger_price_type, OrderTriggerPriceType::MARK);
    assert_eq!(tp.price, Decimal::from(50000));
    assert_eq!(tp.price_type, OrderPriceType::LIMIT);
    assert_eq!(
        tp.settlement.signature.r.to_hex_string(),
        "0x19a043716e5b47bdfa8743e1cad471da3a86dc5a4044a87fb51bea4d61d788c"
    );
    assert_eq!(
        tp.settlement.signature.s.to_hex_string(),
        "0x70db738d6d4896b757e062fec0f3eb8fdcf7d5de23ace3d3c44c1fc9c9c66d4"
    );
    let tp_amounts = tp.debugging_amounts.as_ref().unwrap();
    assert_eq!(tp_amounts.collateral_amount, Decimal::from(50000000));
    assert_eq!(tp_amounts.fee_amount, Decimal::from(25000));
    assert_eq!(tp_amounts.synthetic_amount, Decimal::from(-1000));

    // Stop-loss trigger (opposite side = SELL, price = 39000)
    let sl = order.stop_loss.as_ref().expect("expected stop_loss");
    assert_eq!(sl.trigger_price, Decimal::from(40000));
    assert_eq!(sl.trigger_price_type, OrderTriggerPriceType::MARK);
    assert_eq!(sl.price, Decimal::from(39000));
    assert_eq!(sl.price_type, OrderPriceType::LIMIT);
    assert_eq!(
        sl.settlement.signature.r.to_hex_string(),
        "0xa1d28df388fb5038c2475527667b726ccec821d8362a803702b3a0428ba647"
    );
    assert_eq!(
        sl.settlement.signature.s.to_hex_string(),
        "0x511a2c6a9dc215d965ca08fe2c1533923b2470b1625e1144c70c63b26671086"
    );
    let sl_amounts = sl.debugging_amounts.as_ref().unwrap();
    assert_eq!(sl_amounts.collateral_amount, Decimal::from(39000000));
    assert_eq!(sl_amounts.fee_amount, Decimal::from(19500));
    assert_eq!(sl_amounts.synthetic_amount, Decimal::from(-1000));
}

/// Port of Python test: test_cancel_previous_order
#[test]
fn test_cancel_previous_order() {
    let account = create_trading_account();
    let market = create_btc_usd_market();

    let frozen_time = Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(2024, 1, 5)
            .unwrap()
            .and_hms_micro_opt(1, 8, 56, 860694)
            .unwrap(),
    );
    let expire_time = frozen_time + Duration::days(14);

    let order = create_order_object(
        &account,
        &market,
        Decimal::from_str("0.00100000").unwrap(),
        Decimal::from_str("43445.11680000").unwrap(),
        OrderSide::BUY,
        &TESTNET_CONFIG.starknet_domain,
        false,
        Some("previous_custom_id".to_string()),
        Some(expire_time),
        None,
        TimeInForce::GTT,
        SelfTradeProtectionLevel::ACCOUNT,
        Some(FROZEN_NONCE),
        None,
        None,
        false,
        None,
        None,
        None,
    )
    .expect("create_order_object failed");

    assert_eq!(order.cancel_id.as_deref(), Some("previous_custom_id"));
}

/// Port of Python test: test_external_order_id
#[test]
fn test_external_order_id() {
    let account = create_trading_account();
    let market = create_btc_usd_market();

    let frozen_time = Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(2024, 1, 5)
            .unwrap()
            .and_hms_micro_opt(1, 8, 56, 860694)
            .unwrap(),
    );
    let expire_time = frozen_time + Duration::days(14);

    let order = create_order_object(
        &account,
        &market,
        Decimal::from_str("0.00100000").unwrap(),
        Decimal::from_str("43445.11680000").unwrap(),
        OrderSide::BUY,
        &TESTNET_CONFIG.starknet_domain,
        false,
        None,
        Some(expire_time),
        Some("custom_id".to_string()),
        TimeInForce::GTT,
        SelfTradeProtectionLevel::ACCOUNT,
        Some(FROZEN_NONCE),
        None,
        None,
        false,
        None,
        None,
        None,
    )
    .expect("create_order_object failed");

    assert_eq!(order.id, "custom_id");
}

/// Verify JSON serialization matches the expected API request format.
#[test]
fn test_sell_order_json_serialization() {
    let account = create_trading_account();
    let market = create_btc_usd_market();

    let frozen_time = Utc.from_utc_datetime(
        &NaiveDate::from_ymd_opt(2024, 1, 5)
            .unwrap()
            .and_hms_micro_opt(1, 8, 56, 860694)
            .unwrap(),
    );
    let expire_time = frozen_time + Duration::days(14);

    let order = create_order_object(
        &account,
        &market,
        Decimal::from_str("0.00100000").unwrap(),
        Decimal::from_str("43445.11680000").unwrap(),
        OrderSide::SELL,
        &TESTNET_CONFIG.starknet_domain,
        false,
        None,
        Some(expire_time),
        None,
        TimeInForce::GTT,
        SelfTradeProtectionLevel::ACCOUNT,
        Some(FROZEN_NONCE),
        None,
        None,
        false,
        None,
        None,
        None,
    )
    .unwrap();

    let json: serde_json::Value = serde_json::to_value(&order).unwrap();

    assert_eq!(json["market"], "BTC-USD");
    assert_eq!(json["type"], "LIMIT");
    assert_eq!(json["side"], "SELL");
    assert_eq!(json["timeInForce"], "GTT");
    assert_eq!(json["selfTradeProtectionLevel"], "ACCOUNT");
    assert_eq!(json["expiryEpochMillis"], 1705626536861i64);
    assert_eq!(json["settlement"]["starkKey"], "0x61c5e7e8339b7d56f197f54ea91b776776690e3232313de0f2ecbd0ef76f466");
    assert_eq!(json["settlement"]["collateralPosition"], "10002");
}

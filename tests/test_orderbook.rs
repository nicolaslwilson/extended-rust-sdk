use rust_decimal_macros::dec;

use extended_rust_sdk::orderbook::OrderBook;
use extended_rust_sdk::types::orderbook::{OrderbookQuantityModel, OrderbookUpdateModel};

fn create_dummy_orderbook() -> OrderBook {
    let mut ob = OrderBook::new();
    let snapshot = OrderbookUpdateModel {
        market: "dummy-market".to_string(),
        bid: vec![
            OrderbookQuantityModel { price: dec!(100), qty: dec!(1) },
            OrderbookQuantityModel { price: dec!(99), qty: dec!(2) },
            OrderbookQuantityModel { price: dec!(98), qty: dec!(1) },
        ],
        ask: vec![
            OrderbookQuantityModel { price: dec!(101), qty: dec!(1) },
            OrderbookQuantityModel { price: dec!(102), qty: dec!(2) },
            OrderbookQuantityModel { price: dec!(103), qty: dec!(1) },
        ],
    };
    ob.init_orderbook(&snapshot);
    ob
}

// --- Notional impact tests ---

#[test]
fn test_calculate_impact_partial_buy() {
    let ob = create_dummy_orderbook();
    let notional = dec!(105);
    let result = ob.calculate_price_impact_notional(notional, "BUY").unwrap();

    let expected_amount = dec!(1) + dec!(4) / dec!(102);
    let expected_price = notional / expected_amount;
    assert_eq!(result.amount, expected_amount);
    assert_eq!(result.price, expected_price);
}

#[test]
fn test_calculate_impact_partial_sell() {
    let ob = create_dummy_orderbook();
    let notional = dec!(110);
    let result = ob.calculate_price_impact_notional(notional, "SELL").unwrap();

    let expected_amount = dec!(1) + dec!(10) / dec!(99);
    let expected_price = notional / expected_amount;
    assert_eq!(result.amount, expected_amount);
    assert_eq!(result.price, expected_price);
}

#[test]
fn test_calculate_price_impact_total_match_sell() {
    let ob = create_dummy_orderbook();
    let notional = dec!(199);
    let result = ob.calculate_price_impact_notional(notional, "SELL").unwrap();

    let expected_amount = dec!(2);
    let expected_price = notional / expected_amount;
    assert_eq!(result.amount, expected_amount);
    assert_eq!(result.price, expected_price);
}

#[test]
fn test_calculate_price_impact_total_match_buy() {
    let ob = create_dummy_orderbook();
    let notional = dec!(101) + dec!(2) * dec!(102) + dec!(103);
    let result = ob.calculate_price_impact_notional(notional, "BUY").unwrap();

    let expected_amount = dec!(4);
    let expected_price = notional / expected_amount;
    assert_eq!(result.amount, expected_amount);
    assert_eq!(result.price, expected_price);
}

#[test]
fn test_calculate_price_impact_insufficient_liquidity_bid() {
    let ob = create_dummy_orderbook();
    let result = ob.calculate_price_impact_notional(dec!(1000), "SELL");
    assert!(result.is_none());
}

#[test]
fn test_calculate_price_impact_insufficient_liquidity_ask() {
    let ob = create_dummy_orderbook();
    let result = ob.calculate_price_impact_notional(dec!(1000), "BUY");
    assert!(result.is_none());
}

#[test]
fn test_calculate_price_impact_invalid_notional() {
    let ob = create_dummy_orderbook();
    let result = ob.calculate_price_impact_notional(dec!(-10), "SELL");
    assert!(result.is_none());
}

#[test]
fn test_calculate_price_impact_invalid_side() {
    let ob = create_dummy_orderbook();
    let result = ob.calculate_price_impact_notional(dec!(100), "invalid");
    assert!(result.is_none());
}

// --- Quantity impact tests ---

#[test]
fn test_calculate_qty_impact_partial_buy() {
    let ob = create_dummy_orderbook();
    let qty = dec!(2);
    let result = ob.calculate_price_impact_qty(qty, "BUY").unwrap();

    assert_eq!(result.amount, qty);
    // 1 @101 + 1 @102 = 203, avg = 101.5
    assert_eq!(result.price, dec!(101.5));
}

#[test]
fn test_calculate_qty_impact_partial_sell() {
    let ob = create_dummy_orderbook();
    let qty = dec!(2);
    let result = ob.calculate_price_impact_qty(qty, "SELL").unwrap();

    assert_eq!(result.amount, qty);
    // 1 @100 + 1 @99 = 199, avg = 99.5
    assert_eq!(result.price, dec!(99.5));
}

#[test]
fn test_calculate_qty_impact_total_match_buy() {
    let ob = create_dummy_orderbook();
    let qty = dec!(4);
    let result = ob.calculate_price_impact_qty(qty, "BUY").unwrap();

    assert_eq!(result.amount, qty);
    // 1@101 + 2@102 + 1@103 = 408, avg = 102
    assert_eq!(result.price, dec!(102));
}

#[test]
fn test_calculate_qty_impact_total_match_sell() {
    let ob = create_dummy_orderbook();
    let qty = dec!(4);
    let result = ob.calculate_price_impact_qty(qty, "SELL").unwrap();

    assert_eq!(result.amount, qty);
    // 1@100 + 2@99 + 1@98 = 396, avg = 99
    assert_eq!(result.price, dec!(99));
}

#[test]
fn test_calculate_qty_impact_insufficient_liquidity_buy() {
    let ob = create_dummy_orderbook();
    let result = ob.calculate_price_impact_qty(dec!(5), "BUY");
    assert!(result.is_none());
}

#[test]
fn test_calculate_qty_impact_insufficient_liquidity_sell() {
    let ob = create_dummy_orderbook();
    let result = ob.calculate_price_impact_qty(dec!(5), "SELL");
    assert!(result.is_none());
}

#[test]
fn test_calculate_qty_impact_invalid_qty() {
    let ob = create_dummy_orderbook();
    assert!(ob.calculate_price_impact_qty(dec!(-1), "BUY").is_none());
    assert!(ob.calculate_price_impact_qty(dec!(0), "SELL").is_none());
}

#[test]
fn test_calculate_qty_impact_invalid_side() {
    let ob = create_dummy_orderbook();
    let result = ob.calculate_price_impact_qty(dec!(1), "INVALID_SIDE");
    assert!(result.is_none());
}

// --- Basic orderbook tests ---

#[test]
fn test_best_bid_ask_spread() {
    let ob = create_dummy_orderbook();
    assert_eq!(ob.best_bid().unwrap().price, dec!(100));
    assert_eq!(ob.best_ask().unwrap().price, dec!(101));
    assert_eq!(ob.spread(), Some(dec!(1)));
}

#[test]
fn test_delta_update() {
    let mut ob = create_dummy_orderbook();

    // Add quantity at existing price level
    let delta = OrderbookUpdateModel {
        market: "dummy-market".to_string(),
        bid: vec![OrderbookQuantityModel { price: dec!(100), qty: dec!(2) }],
        ask: vec![],
    };
    ob.update_orderbook(&delta);
    assert_eq!(ob.best_bid().unwrap().amount, dec!(3)); // 1 + 2

    // Remove quantity to zero (removes level)
    let delta2 = OrderbookUpdateModel {
        market: "dummy-market".to_string(),
        bid: vec![OrderbookQuantityModel { price: dec!(100), qty: dec!(-3) }],
        ask: vec![],
    };
    ob.update_orderbook(&delta2);
    assert_eq!(ob.best_bid().unwrap().price, dec!(99)); // 100 removed
}

use extended_rust_sdk::{
    OrderSide, PerpetualTradingClient, SelfTradeProtectionLevel,
    StarkPerpetualAccount, TimeInForce, TESTNET_CONFIG,
};
use rust_decimal_macros::dec;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Set up the account
    let account = StarkPerpetualAccount::new(
        10002,
        "0x_YOUR_PRIVATE_KEY",
        "0x_YOUR_PUBLIC_KEY",
        "your_api_key",
    );

    // 2. Create the trading client
    let client =
        PerpetualTradingClient::new(TESTNET_CONFIG.clone(), Some(account))?;

    // 3. Place an order using the convenience method
    let result = client
        .place_order(
            "BTC-USD",
            dec!(0.001),       // amount
            dec!(43445.0),     // price
            OrderSide::BUY,
            false,             // post_only
            None,              // previous_order_id
            None,              // expire_time (default: 1 hour)
            TimeInForce::GTT,
            SelfTradeProtectionLevel::ACCOUNT,
            None,              // external_id
            None,              // builder_fee
            None,              // builder_id
            false,             // reduce_only
            None,              // tp_sl_type
            None,              // take_profit
            None,              // stop_loss
        )
        .await?;

    println!("Order placed: {:?}", result.data);

    // 4. Or use the lower-level approach: fetch market + build order manually
    let markets = client.markets_info().get_markets(None).await?;
    let btc_market = markets
        .data
        .as_ref()
        .unwrap()
        .iter()
        .find(|m| m.name == "BTC-USD")
        .unwrap();

    println!("BTC-USD market: {}", btc_market.name);
    println!(
        "  Last price: {}",
        btc_market.market_stats.last_price
    );

    Ok(())
}

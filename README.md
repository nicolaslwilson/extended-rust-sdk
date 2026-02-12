# Extended Rust SDK

Rust SDK for the [Extended](https://extended.exchange) crypto perpetual exchange.

## Features

- **REST API** -- full trading client with order management, account operations, market data, and testnet utilities
- **WebSocket streaming** -- real-time orderbooks, trades, funding rates, candles, and authenticated account updates
- **Order signing** -- STARK cryptographic order construction and signing via [rust-crypto-lib-base](https://github.com/x10xchange/rust-crypto-lib-base)
- **Onboarding** -- L2 key derivation from Ethereum L1 signatures (EIP-712) and account registration
- **Local orderbook** -- maintained from WebSocket deltas with best bid/ask and price impact calculations
- **Builder pattern** -- ergonomic `OrderBuilder` for constructing signed orders

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
extended-rust-sdk = { git = "https://github.com/x10xchange/extended-rust-sdk" }
```

## Quick start

### Place a limit order

```rust
use extended_rust_sdk::{
    OrderSide, PerpetualTradingClient, SelfTradeProtectionLevel,
    StarkPerpetualAccount, TimeInForce, TESTNET_CONFIG,
};
use rust_decimal_macros::dec;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let account = StarkPerpetualAccount::new(
        10002,
        "0x_YOUR_PRIVATE_KEY",
        "0x_YOUR_PUBLIC_KEY",
        "your_api_key",
    );

    let client = PerpetualTradingClient::new(TESTNET_CONFIG.clone(), Some(account))?;

    let result = client
        .place_order(
            "BTC-USD",
            dec!(0.001),           // amount
            dec!(43445.0),         // price
            OrderSide::BUY,
            false,                 // post_only
            None,                  // previous_order_id
            None,                  // expire_time (default: 1 hour)
            TimeInForce::GTT,
            SelfTradeProtectionLevel::ACCOUNT,
            None, None, None,      // external_id, builder_fee, builder_id
            false,                 // reduce_only
            None, None, None,      // tp_sl_type, take_profit, stop_loss
        )
        .await?;

    println!("Order placed: {:?}", result.data);
    Ok(())
}
```

### Using the OrderBuilder

```rust
use extended_rust_sdk::{OrderBuilder, OrderSide, TESTNET_CONFIG};
use rust_decimal_macros::dec;

let order = OrderBuilder::new(&account, &market, OrderSide::BUY, dec!(0.001), dec!(43445.0), &TESTNET_CONFIG.starknet_domain)
    .post_only(true)
    .time_in_force(extended_rust_sdk::TimeInForce::GTT)
    .build()?;
```

### Stream orderbook updates

```rust
use extended_rust_sdk::{OrderBook, PerpetualStreamClient, TESTNET_CONFIG};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream_client = PerpetualStreamClient::new(&TESTNET_CONFIG.stream_url);
    let mut conn = stream_client
        .subscribe_to_orderbooks(Some("BTC-USD"), Some(10))
        .await?;

    let mut orderbook = OrderBook::new();
    let mut is_first = true;

    while let Some(msg) = conn.next().await {
        let update = msg?;
        if let Some(data) = &update.data {
            if is_first {
                orderbook.init_orderbook(data);
                is_first = false;
            } else {
                orderbook.update_orderbook(data);
            }

            if let (Some(bid), Some(ask)) = (orderbook.best_bid(), orderbook.best_ask()) {
                println!("Bid: {} @ {} | Ask: {} @ {}", bid.amount, bid.price, ask.amount, ask.price);
            }
        }
    }
    Ok(())
}
```

### Onboarding (account registration)

```rust
use extended_rust_sdk::{UserClient, TESTNET_CONFIG};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let eth_private_key = "0x_YOUR_ETH_PRIVATE_KEY".to_string();

    let pk = eth_private_key.clone();
    let user_client = UserClient::new(TESTNET_CONFIG.clone(), move || pk.clone())?;

    // Derives L2 keys from L1, signs EIP-712 payloads, registers on exchange
    let onboarded = user_client.onboard(None).await?;
    println!("Account: {:?}", onboarded.account);

    // Create an API key for trading
    let api_key = user_client
        .create_account_api_key(&onboarded.account, Some("My Bot"))
        .await?;
    println!("API key: {}", api_key);
    Ok(())
}
```

## API overview

### PerpetualTradingClient

The main REST client, organized into modules:

| Module | Access | Endpoints |
|--------|--------|-----------|
| **info** | `client.info()` | Exchange settings |
| **markets_info** | `client.markets_info()` | Markets, candles, funding rates, orderbook snapshots |
| **account** | `client.account()` | Balance, positions, open orders, order history, trades, fees, leverage, transfers, withdrawals |
| **orders** | `client.orders()` | Place order, cancel order, mass cancel, dead man's switch |
| **testnet** | `client.testnet()` | Claim testing funds |

### PerpetualStreamClient

WebSocket subscriptions:

| Method | Data type | Auth |
|--------|-----------|------|
| `subscribe_to_orderbooks` | `OrderbookUpdateModel` | No |
| `subscribe_to_public_trades` | `Vec<PublicTradeModel>` | No |
| `subscribe_to_funding_rates` | `FundingRateModel` | No |
| `subscribe_to_candles` | `Vec<CandleModel>` | No |
| `subscribe_to_account_updates` | `AccountStreamDataModel` | Yes (API key) |

### UserClient

Account onboarding and management:

| Method | Description |
|--------|-------------|
| `onboard` | Register a new account (derives L2 keys from Ethereum signature) |
| `onboard_subaccount` | Create a subaccount |
| `get_accounts` | List all accounts |
| `create_account_api_key` | Generate an API key |

## Configuration

Built-in configs for testnet and mainnet:

```rust
use extended_rust_sdk::{TESTNET_CONFIG, MAINNET_CONFIG};

// Testnet: api.starknet.sepolia.extended.exchange
let client = PerpetualTradingClient::new(TESTNET_CONFIG.clone(), Some(account))?;

// Mainnet: api.starknet.extended.exchange
let client = PerpetualTradingClient::new(MAINNET_CONFIG.clone(), Some(account))?;
```

## Examples

See the [`examples/`](examples/) directory:

- [`place_limit_order.rs`](examples/place_limit_order.rs) -- basic order placement
- [`stream_orderbook.rs`](examples/stream_orderbook.rs) -- WebSocket orderbook streaming with local orderbook
- [`onboarding.rs`](examples/onboarding.rs) -- full onboarding flow

Run with:

```bash
cargo run --example place_limit_order
cargo run --example stream_orderbook
cargo run --example onboarding
```

## API documentation

- [Extended API docs](https://api.docs.extended.exchange)
- [Python SDK](https://github.com/x10xchange/python_sdk) (reference implementation)
- [rust-crypto-lib-base](https://github.com/x10xchange/rust-crypto-lib-base) (cryptographic primitives)

## License

MIT

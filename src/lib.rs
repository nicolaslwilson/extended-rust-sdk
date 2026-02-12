#![allow(clippy::too_many_arguments)]
#![allow(clippy::result_large_err)]

pub mod amounts;
pub mod config;
pub mod error;
pub mod http;
pub mod order_builder;
pub mod order_settlement;
pub mod orderbook;
pub mod stream;
pub mod trading_client;
pub mod transfer_builder;
pub mod types;
pub mod user_client;
pub mod utils;
pub mod withdrawal_builder;

pub use config::{SDK_VERSION, USER_AGENT};
pub use error::{X10Error, X10Result};
pub use order_builder::{create_order_object, OrderBuilder, OrderTpslTriggerParam};
pub use orderbook::OrderBook;
pub use stream::PerpetualStreamClient;
pub use trading_client::PerpetualTradingClient;
pub use types::account::StarkPerpetualAccount;
pub use types::config::{EndpointConfig, StarknetDomain, MAINNET_CONFIG, TESTNET_CONFIG};
pub use types::enums::*;
pub use user_client::UserClient;

use std::sync::LazyLock;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingFeeModel {
    pub market: String,
    pub maker_fee_rate: Decimal,
    pub taker_fee_rate: Decimal,
    pub builder_fee_rate: Decimal,
}

pub static DEFAULT_FEES: LazyLock<TradingFeeModel> = LazyLock::new(|| TradingFeeModel {
    market: "BTC-USD".into(),
    maker_fee_rate: dec!(2) / dec!(10000),   // 0.0002
    taker_fee_rate: dec!(5) / dec!(10000),   // 0.0005
    builder_fee_rate: dec!(0),
});

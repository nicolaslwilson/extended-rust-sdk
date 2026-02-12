use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceModel {
    pub collateral_name: String,
    pub balance: Decimal,
    pub equity: Decimal,
    pub available_for_trade: Decimal,
    pub available_for_withdrawal: Decimal,
    pub unrealised_pnl: Decimal,
    pub initial_margin: Decimal,
    pub margin_ratio: Decimal,
    pub updated_time: i64,
}

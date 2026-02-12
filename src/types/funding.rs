use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRateModel {
    #[serde(alias = "m")]
    pub market: String,
    #[serde(alias = "f", rename = "f")]
    pub funding_rate: Decimal,
    #[serde(alias = "T")]
    pub timestamp: i64,
}

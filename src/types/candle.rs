use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleModel {
    #[serde(alias = "o")]
    pub open: Decimal,
    #[serde(alias = "l")]
    pub low: Decimal,
    #[serde(alias = "h")]
    pub high: Decimal,
    #[serde(alias = "c")]
    pub close: Decimal,
    #[serde(alias = "v", skip_serializing_if = "Option::is_none")]
    pub volume: Option<Decimal>,
    #[serde(alias = "T")]
    pub timestamp: i64,
}

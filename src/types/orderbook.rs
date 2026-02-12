use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookQuantityModel {
    #[serde(alias = "q")]
    pub qty: Decimal,
    #[serde(alias = "p")]
    pub price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookUpdateModel {
    #[serde(alias = "m")]
    pub market: String,
    #[serde(alias = "b")]
    pub bid: Vec<OrderbookQuantityModel>,
    #[serde(alias = "a")]
    pub ask: Vec<OrderbookQuantityModel>,
}

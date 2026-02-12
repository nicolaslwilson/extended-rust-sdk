use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::asset::Asset;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskFactorConfig {
    pub upper_bound: Decimal,
    pub risk_factor: Decimal,
}

impl RiskFactorConfig {
    pub fn max_leverage(&self) -> Decimal {
        (Decimal::ONE / self.risk_factor)
            .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointNearestEven)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketStatsModel {
    pub daily_volume: Decimal,
    pub daily_volume_base: Decimal,
    pub daily_price_change: Decimal,
    pub daily_low: Decimal,
    pub daily_high: Decimal,
    pub last_price: Decimal,
    pub ask_price: Decimal,
    pub bid_price: Decimal,
    pub mark_price: Decimal,
    pub index_price: Decimal,
    pub funding_rate: Decimal,
    pub next_funding_rate: i64,
    pub open_interest: Decimal,
    pub open_interest_base: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingConfigModel {
    pub min_order_size: Decimal,
    pub min_order_size_change: Decimal,
    pub min_price_change: Decimal,
    pub max_market_order_value: Decimal,
    pub max_limit_order_value: Decimal,
    pub max_position_value: Decimal,
    pub max_leverage: Decimal,
    pub max_num_orders: Decimal,
    pub limit_price_cap: Decimal,
    pub limit_price_floor: Decimal,
    pub risk_factor_config: Vec<RiskFactorConfig>,
}

impl TradingConfigModel {
    pub fn price_precision(&self) -> u32 {
        self.min_price_change.scale()
    }

    pub fn quantity_precision(&self) -> u32 {
        self.min_order_size_change.scale()
    }

    pub fn max_leverage_for_position_value(&self, position_value: Decimal) -> Decimal {
        self.risk_factor_config
            .iter()
            .find(|x| x.upper_bound >= position_value)
            .map(|x| x.max_leverage())
            .unwrap_or(Decimal::ZERO)
    }

    pub fn max_position_value_for_leverage(&self, leverage: Decimal) -> Decimal {
        self.risk_factor_config
            .iter()
            .rev()
            .find(|x| x.max_leverage() >= leverage)
            .map(|x| x.upper_bound)
            .unwrap_or(Decimal::ZERO)
    }

    pub fn round_order_size(&self, order_size: Decimal) -> Decimal {
        let steps = order_size / self.min_order_size_change;
        let rounded = steps.round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
        rounded * self.min_order_size_change
    }

    pub fn calculate_order_size_from_value(&self, order_value: Decimal, order_price: Decimal) -> Decimal {
        let order_size = order_value / order_price;
        if order_size > Decimal::ZERO {
            self.round_order_size(order_size)
        } else {
            Decimal::ZERO
        }
    }

    pub fn round_price(&self, price: Decimal) -> Decimal {
        price.round_dp(self.price_precision())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L2ConfigModel {
    #[serde(rename = "type")]
    pub config_type: String,
    pub collateral_id: String,
    pub collateral_resolution: i64,
    pub synthetic_id: String,
    pub synthetic_resolution: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketModel {
    pub name: String,
    pub asset_name: String,
    pub asset_precision: u32,
    pub collateral_asset_name: String,
    pub collateral_asset_precision: u32,
    pub active: bool,
    pub market_stats: MarketStatsModel,
    pub trading_config: TradingConfigModel,
    pub l2_config: L2ConfigModel,
}

impl MarketModel {
    pub fn synthetic_asset(&self) -> Asset {
        Asset {
            id: 1,
            name: self.asset_name.clone(),
            precision: self.asset_precision,
            active: self.active,
            is_collateral: false,
            settlement_external_id: self.l2_config.synthetic_id.clone(),
            settlement_resolution: self.l2_config.synthetic_resolution,
            l1_external_id: String::new(),
            l1_resolution: 0,
        }
    }

    pub fn collateral_asset(&self) -> Asset {
        Asset {
            id: 2,
            name: self.collateral_asset_name.clone(),
            precision: self.collateral_asset_precision,
            active: self.active,
            is_collateral: true,
            settlement_external_id: self.l2_config.collateral_id.clone(),
            settlement_resolution: self.l2_config.collateral_resolution,
            l1_external_id: String::new(),
            l1_resolution: 0,
        }
    }
}

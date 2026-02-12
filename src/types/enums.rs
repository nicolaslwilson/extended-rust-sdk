#![allow(non_camel_case_types)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInForce {
    GTT,
    IOC,
    FOK,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderSide {
    BUY,
    SELL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderType {
    LIMIT,
    CONDITIONAL,
    MARKET,
    TPSL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderTpslType {
    ORDER,
    POSITION,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    UNKNOWN,
    NEW,
    UNTRIGGERED,
    PARTIALLY_FILLED,
    FILLED,
    CANCELLED,
    EXPIRED,
    REJECTED,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatusReason {
    UNKNOWN,
    NONE,
    UNKNOWN_MARKET,
    DISABLED_MARKET,
    NOT_ENOUGH_FUNDS,
    NO_LIQUIDITY,
    INVALID_FEE,
    INVALID_QTY,
    INVALID_PRICE,
    INVALID_VALUE,
    UNKNOWN_ACCOUNT,
    SELF_TRADE_PROTECTION,
    POST_ONLY_FAILED,
    REDUCE_ONLY_FAILED,
    INVALID_EXPIRE_TIME,
    POSITION_TPSL_CONFLICT,
    INVALID_LEVERAGE,
    PREV_ORDER_NOT_FOUND,
    PREV_ORDER_TRIGGERED,
    TPSL_OTHER_SIDE_FILLED,
    PREV_ORDER_CONFLICT,
    ORDER_REPLACED,
    POST_ONLY_MODE,
    REDUCE_ONLY_MODE,
    TRADING_OFF_MODE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderTriggerPriceType {
    UNKNOWN,
    MARK,
    INDEX,
    LAST,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderTriggerDirection {
    UNKNOWN,
    UP,
    DOWN,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderPriceType {
    UNKNOWN,
    MARKET,
    LIMIT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelfTradeProtectionLevel {
    DISABLED,
    ACCOUNT,
    CLIENT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionSide {
    LONG,
    SHORT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionStatus {
    OPENED,
    CLOSED,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExitType {
    TRADE,
    LIQUIDATION,
    ADL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TradeType {
    TRADE,
    LIQUIDATION,
    DELEVERAGE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetOperationType {
    CLAIM,
    DEPOSIT,
    FAST_WITHDRAWAL,
    SLOW_WITHDRAWAL,
    TRANSFER,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetOperationStatus {
    UNKNOWN,
    CREATED,
    IN_PROGRESS,
    REJECTED,
    READY_FOR_CLAIM,
    COMPLETED,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResponseStatus {
    OK,
    ERROR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StreamDataType {
    UNKNOWN,
    BALANCE,
    DELTA,
    DEPOSIT,
    ORDER,
    POSITION,
    SNAPSHOT,
    TRADE,
    TRANSFER,
    WITHDRAWAL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CandleType {
    #[serde(rename = "trades")]
    Trades,
    #[serde(rename = "mark-prices")]
    MarkPrices,
    #[serde(rename = "index-prices")]
    IndexPrices,
}

impl std::fmt::Display for CandleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CandleType::Trades => write!(f, "trades"),
            CandleType::MarkPrices => write!(f, "mark-prices"),
            CandleType::IndexPrices => write!(f, "index-prices"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CandleInterval {
    PT1M,
    PT5M,
    PT15M,
    PT30M,
    PT1H,
    PT2H,
    PT4H,
    P1D,
}

impl std::fmt::Display for CandleInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CandleInterval::PT1M => "PT1M",
            CandleInterval::PT5M => "PT5M",
            CandleInterval::PT15M => "PT15M",
            CandleInterval::PT30M => "PT30M",
            CandleInterval::PT1H => "PT1H",
            CandleInterval::PT2H => "PT2H",
            CandleInterval::PT4H => "PT4H",
            CandleInterval::P1D => "P1D",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::BUY => write!(f, "BUY"),
            OrderSide::SELL => write!(f, "SELL"),
        }
    }
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::LIMIT => write!(f, "LIMIT"),
            OrderType::CONDITIONAL => write!(f, "CONDITIONAL"),
            OrderType::MARKET => write!(f, "MARKET"),
            OrderType::TPSL => write!(f, "TPSL"),
        }
    }
}

impl std::fmt::Display for PositionSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionSide::LONG => write!(f, "LONG"),
            PositionSide::SHORT => write!(f, "SHORT"),
        }
    }
}

impl std::fmt::Display for TradeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeType::TRADE => write!(f, "TRADE"),
            TradeType::LIQUIDATION => write!(f, "LIQUIDATION"),
            TradeType::DELEVERAGE => write!(f, "DELEVERAGE"),
        }
    }
}

impl std::fmt::Display for AssetOperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetOperationType::CLAIM => write!(f, "CLAIM"),
            AssetOperationType::DEPOSIT => write!(f, "DEPOSIT"),
            AssetOperationType::FAST_WITHDRAWAL => write!(f, "FAST_WITHDRAWAL"),
            AssetOperationType::SLOW_WITHDRAWAL => write!(f, "SLOW_WITHDRAWAL"),
            AssetOperationType::TRANSFER => write!(f, "TRANSFER"),
        }
    }
}

impl std::fmt::Display for AssetOperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetOperationStatus::UNKNOWN => write!(f, "UNKNOWN"),
            AssetOperationStatus::CREATED => write!(f, "CREATED"),
            AssetOperationStatus::IN_PROGRESS => write!(f, "IN_PROGRESS"),
            AssetOperationStatus::REJECTED => write!(f, "REJECTED"),
            AssetOperationStatus::READY_FOR_CLAIM => write!(f, "READY_FOR_CLAIM"),
            AssetOperationStatus::COMPLETED => write!(f, "COMPLETED"),
        }
    }
}

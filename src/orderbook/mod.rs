use std::collections::BTreeMap;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::types::orderbook::OrderbookUpdateModel;

/// Epsilon for notional/qty comparisons to handle Decimal rounding artifacts.
const EPSILON: Decimal = dec!(0.0000000000000000000000001);

#[derive(Debug, Clone)]
pub struct OrderBookEntry {
    pub price: Decimal,
    pub amount: Decimal,
}

#[derive(Debug, Clone)]
pub struct ImpactDetails {
    pub price: Decimal,
    pub amount: Decimal,
}

/// Local orderbook maintained from WebSocket snapshots and deltas.
pub struct OrderBook {
    /// Bids sorted by price (ascending key; highest bid = last entry).
    bid_prices: BTreeMap<Decimal, OrderBookEntry>,
    /// Asks sorted by price (ascending key; lowest ask = first entry).
    ask_prices: BTreeMap<Decimal, OrderBookEntry>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bid_prices: BTreeMap::new(),
            ask_prices: BTreeMap::new(),
        }
    }

    /// Initialize the orderbook from a snapshot (replaces all data).
    pub fn init_orderbook(&mut self, data: &OrderbookUpdateModel) {
        self.bid_prices.clear();
        self.ask_prices.clear();

        for bid in &data.bid {
            self.bid_prices.insert(
                bid.price,
                OrderBookEntry {
                    price: bid.price,
                    amount: bid.qty,
                },
            );
        }

        for ask in &data.ask {
            self.ask_prices.insert(
                ask.price,
                OrderBookEntry {
                    price: ask.price,
                    amount: ask.qty,
                },
            );
        }
    }

    /// Apply a delta update to the orderbook.
    pub fn update_orderbook(&mut self, data: &OrderbookUpdateModel) {
        for bid in &data.bid {
            if let Some(entry) = self.bid_prices.get_mut(&bid.price) {
                entry.amount += bid.qty;
                if entry.amount == Decimal::ZERO {
                    self.bid_prices.remove(&bid.price);
                }
            } else {
                self.bid_prices.insert(
                    bid.price,
                    OrderBookEntry {
                        price: bid.price,
                        amount: bid.qty,
                    },
                );
            }
        }

        for ask in &data.ask {
            if let Some(entry) = self.ask_prices.get_mut(&ask.price) {
                entry.amount += ask.qty;
                if entry.amount == Decimal::ZERO {
                    self.ask_prices.remove(&ask.price);
                }
            } else {
                self.ask_prices.insert(
                    ask.price,
                    OrderBookEntry {
                        price: ask.price,
                        amount: ask.qty,
                    },
                );
            }
        }
    }

    /// Get the best (highest) bid.
    pub fn best_bid(&self) -> Option<&OrderBookEntry> {
        self.bid_prices.values().next_back()
    }

    /// Get the best (lowest) ask.
    pub fn best_ask(&self) -> Option<&OrderBookEntry> {
        self.ask_prices.values().next()
    }

    /// Get the spread (best_ask - best_bid).
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.price - bid.price),
            _ => None,
        }
    }

    /// Calculate the price impact for a given notional amount.
    pub fn calculate_price_impact_notional(
        &self,
        notional: Decimal,
        side: &str,
    ) -> Option<ImpactDetails> {
        if notional <= Decimal::ZERO {
            return None;
        }
        match side {
            "SELL" => {
                if self.bid_prices.is_empty() {
                    return None;
                }
                self.price_impact_notional(notional, self.bid_prices.iter().rev())
            }
            "BUY" => {
                if self.ask_prices.is_empty() {
                    return None;
                }
                self.price_impact_notional(notional, self.ask_prices.iter())
            }
            _ => None,
        }
    }

    /// Calculate the price impact for a given quantity.
    pub fn calculate_price_impact_qty(
        &self,
        qty: Decimal,
        side: &str,
    ) -> Option<ImpactDetails> {
        if qty <= Decimal::ZERO {
            return None;
        }
        match side {
            "SELL" => {
                if self.bid_prices.is_empty() {
                    return None;
                }
                self.price_impact_qty(qty, self.bid_prices.iter().rev())
            }
            "BUY" => {
                if self.ask_prices.is_empty() {
                    return None;
                }
                self.price_impact_qty(qty, self.ask_prices.iter())
            }
            _ => None,
        }
    }

    fn price_impact_notional<'a, I>(
        &self,
        notional: Decimal,
        levels: I,
    ) -> Option<ImpactDetails>
    where
        I: Iterator<Item = (&'a Decimal, &'a OrderBookEntry)>,
    {
        let mut remaining = notional;
        let mut total_amount = Decimal::ZERO;
        let mut weighted_sum = Decimal::ZERO;

        for (_price, entry) in levels {
            if remaining <= Decimal::ZERO {
                break;
            }
            if entry.amount <= Decimal::ZERO {
                continue;
            }
            let amount_to_purchase = std::cmp::min(remaining / entry.price, entry.amount);
            let take = amount_to_purchase;
            let spent = take * entry.price;
            weighted_sum += take * entry.price;
            total_amount += take;
            remaining -= spent;
        }

        if remaining > EPSILON {
            return None;
        }
        let average_price = weighted_sum / total_amount;
        Some(ImpactDetails {
            price: average_price,
            amount: total_amount,
        })
    }

    fn price_impact_qty<'a, I>(
        &self,
        qty: Decimal,
        levels: I,
    ) -> Option<ImpactDetails>
    where
        I: Iterator<Item = (&'a Decimal, &'a OrderBookEntry)>,
    {
        let mut remaining_qty = qty;
        let mut total_amount = Decimal::ZERO;
        let mut total_spent = Decimal::ZERO;

        for (_price, entry) in levels {
            if remaining_qty <= Decimal::ZERO {
                break;
            }
            if entry.amount <= Decimal::ZERO {
                continue;
            }
            let take = std::cmp::min(remaining_qty, entry.amount);
            total_spent += take * entry.price;
            total_amount += take;
            remaining_qty -= take;
        }

        if remaining_qty > EPSILON {
            return None;
        }
        let average_price = total_spent / total_amount;
        Some(ImpactDetails {
            price: average_price,
            amount: total_amount,
        })
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

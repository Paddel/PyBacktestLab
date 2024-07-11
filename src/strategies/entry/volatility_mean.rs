use crate::{algorithms::Algorithms, evaluation::signal::SignalResult, prices::price::{Ohlc, Price, Tick}};

use super::entry::Entry;

const KEY_PRICE_TO_OPEN: &str = "price_to_open";

pub struct VolatilityMean {
    entry_factor: f32,
    vol_timeframe: f32,
}

impl VolatilityMean {
    pub fn new(entry_factor: f32, vol_timeframe: f32) -> VolatilityMean {
        VolatilityMean {
            entry_factor,
            vol_timeframe,
        }
    }
}

impl Entry for VolatilityMean {
    fn on_init(&self, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) {
        let price_type  = if signal_result.signal.action == "buy" {(Tick::Ask, Ohlc::Close)} else {(Tick::Bid, Ohlc::Close)};
        let volatility = Algorithms::calculate_volatility(&price_type, prices, self.vol_timeframe);
        let mean = Algorithms::calculate_mean(&price_type, prices, self.vol_timeframe as f32);
        let price_to_open = mean * Algorithms::calculate_volatility_factor(signal_result.signal.action.as_str(), volatility, self.entry_factor);
        signal_result.position.strategy_attributes.insert(KEY_PRICE_TO_OPEN.to_string(), price_to_open);
    }
    
    fn check_entry(&self, signal_result: &SignalResult, prices: &[Box<dyn Price>]) -> Option<f32> {
        if signal_result.signal.action == "buy" {
            let price = prices.last().unwrap().get(&(Tick::Ask, Ohlc::Low));
            let border = *signal_result.position.strategy_attributes.get(KEY_PRICE_TO_OPEN).unwrap();
            if price <= border {
                Some(border)
            } else {
                None
            }
        } else {
            let price = prices.last().unwrap().get(&(Tick::Bid, Ohlc::High));
            let border = *signal_result.position.strategy_attributes.get(KEY_PRICE_TO_OPEN).unwrap();
            if price >= border {
                Some(border)
            } else {
                None
            }
        }
    }
}
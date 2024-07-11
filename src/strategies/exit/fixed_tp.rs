use crate::{algorithms::Algorithms, evaluation::signal::SignalResult, prices::price::{Ohlc, Price, Tick}};

use super::exit::Exit;

const KEY_TAKE_PROFIT: &str = "take_profit";

pub struct FixedTP {
    tp_factor: f32,
    vol_timeframe: f32,
}

impl FixedTP {
    pub fn new(tp_factor: f32, vol_timeframe: f32) -> FixedTP {
        FixedTP {
            tp_factor,
            vol_timeframe,
        }
    }
}

impl Exit for FixedTP {
    fn on_open(&self, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) {
        let price_type = if signal_result.signal.action == "buy" {(Tick::Bid, Ohlc::Close)} else {(Tick::Ask, Ohlc::Close)};
        let volatility = Algorithms::calculate_volatility(&price_type, prices, self.vol_timeframe);
        let take_profit_given = {
            if signal_result.signal.take_profit.len() > 0 {
                signal_result.signal.take_profit[0]
            } else {
                // signal_result.position.closed = true;
                let price = prices.last().unwrap().get(&price_type);
                if signal_result.signal.action == "buy" {
                    price + (price - signal_result.signal.stop_loss) * 2.0
                } else {
                    price - (signal_result.signal.stop_loss - price) * 2.0
                }
                
            }
        };
        let take_profit = take_profit_given * Algorithms::calculate_volatility_factor(signal_result.signal.action.as_str(), volatility, self.tp_factor);
        signal_result.position.strategy_attributes.insert(KEY_TAKE_PROFIT.to_string(), take_profit);
    }
    
    fn check_exit(&self, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) -> Option<f32> {
        let take_profit = signal_result.position.strategy_attributes.get(KEY_TAKE_PROFIT).expect("Take profit not set");
        if signal_result.signal.action == "buy" {
            let price = prices.last().unwrap().get(&(Tick::Bid, Ohlc::High));
            let border = *take_profit;
            if price >= border {
                Some(border)
            } else {
                None
            }
        } else {
            let price = prices.last().unwrap().get(&(Tick::Ask, Ohlc::Low));
            let border = *take_profit;
            if price <= border {
                Some(border)
            } else {
                None
            }
        }
    }
}
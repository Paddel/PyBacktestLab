use crate::{algorithms::Algorithms, evaluation::signal::SignalResult, prices::price::{Ohlc, Price, Tick}};

use super::entry::Entry;

pub struct Bollinger {
    std_dev_factor: f32,
    period_minutes: i32,
}

impl Bollinger {
    pub fn new(std_dev_factor: f32, period_minutes: i32) -> Bollinger {
        Bollinger {
            std_dev_factor,
            period_minutes,
        }
    }
}

impl Entry for Bollinger {
    fn on_init(&self, _signal_result: &mut SignalResult, _prices: &[Box<dyn Price>]) { }
    
    fn check_entry(&self, signal_result: &SignalResult, prices: &[Box<dyn Price>]) -> Option<f32> {
        let price_type = if signal_result.signal.action == "buy" {(Tick::Ask, Ohlc::Close)} else {(Tick::Bid, Ohlc::Close)};
        let mean = Algorithms::calculate_mean(&price_type, prices, self.period_minutes as f32);
        let std_dev = Algorithms::calculate_standard_deviation(&price_type, prices, self.period_minutes as f32, mean);
        if signal_result.signal.action == "buy" {
            let last_price = prices.last().unwrap().get(&(Tick::Ask, Ohlc::Low));
            let border = mean - std_dev * self.std_dev_factor;
            if last_price <= border {
                Some(border)
            } else {
                None
            }
        } else {
            let last_price = prices.last().unwrap().get(&(Tick::Bid, Ohlc::High));
            let border = mean + std_dev * self.std_dev_factor;
            if last_price >= border {
                Some(border)
            } else {
                None
            }
        }
    }
}
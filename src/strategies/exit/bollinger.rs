use crate::{algorithms::Algorithms, evaluation::signal::SignalResult, prices::price::{Ohlc, Price, Tick}};

use super::exit::Exit;

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

impl Exit for Bollinger {
    fn on_open(&self, _signal_result: &mut SignalResult, _prices: &[Box<dyn Price>]) { }
    
    fn check_exit(&self, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) -> Option<f32> {
        let price_type = if signal_result.signal.action == "buy" {(Tick::Bid, Ohlc::Close)} else {(Tick::Ask, Ohlc::Close)};
        let mean = Algorithms::calculate_mean(&price_type, prices, self.period_minutes as f32);
        let std_dev = Algorithms::calculate_standard_deviation(&price_type, prices, self.period_minutes as f32, mean);
        let index_last_second = {
            let mut index = prices.len() - 1;
            while index > 0 && prices.last().unwrap().ts() - prices[index].ts() < 60 * 1000 {
                index -= 1;
            }
            index
        };

        let price_type_close = if signal_result.signal.action == "buy" {(Tick::Bid, Ohlc::High)} else {(Tick::Ask, Ohlc::Low)};
        let mean_1m = Algorithms::calculate_mean(&price_type_close, prices, 2 as f32);
        let mean_1m_last = Algorithms::calculate_mean(&price_type_close, &prices[index_last_second..], 1 as f32);

        if signal_result.signal.action == "buy" {
            let border = mean + std_dev * self.std_dev_factor;
            if mean_1m < border && mean_1m_last >= border {
                Some(border)
            } else {
                None
            }
        } else {
            let border = mean - std_dev * self.std_dev_factor;
            if mean_1m > border && mean_1m_last <= border {
                Some(border)
            } else {
                None
            }
        }
    }
}
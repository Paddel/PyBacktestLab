use crate::{evaluation::signal::SignalResult, prices::price::{Ohlc, Price, Tick}};

use super::entry::Entry;

pub struct Immediate {
}

impl Immediate {
    pub fn new() -> Immediate {
        Immediate {}
    }
}

impl Entry for Immediate {
    fn on_init(&self, _signal_result: &mut SignalResult, _prices: &[Box<dyn Price>]) { }
    
    fn check_entry(&self, signal_result: &SignalResult, prices: &[Box<dyn Price>]) -> Option<f32> {
        let price_type = if signal_result.signal.action == "buy" {(Tick::Ask, Ohlc::Close)} else {(Tick::Bid, Ohlc::Close)};
        let last_price = prices.last().unwrap().get(&price_type);
        Some(last_price)
    }
}
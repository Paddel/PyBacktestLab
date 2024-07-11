use crate::{evaluation::signal::SignalResult, prices::price::{Ohlc, Price, Tick}};

use super::exit::Exit;

const KEY_EXTREME: &str = "extreme";
const KEY_SL: &str = "sl";

pub struct TrailingStop {
    sl_factor: f32,
}

impl TrailingStop {
    pub fn new(sl_factor: f32) -> TrailingStop {
        TrailingStop {
            sl_factor,
        }
    }
}

impl Exit for TrailingStop {
    fn on_open(&self, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) {
        let price_type = if signal_result.signal.action == "buy" {(Tick::Bid, Ohlc::High)} else {(Tick::Ask, Ohlc::Low)};
        let price = prices.last().unwrap().get(&price_type);
        signal_result.position.strategy_attributes.insert(KEY_EXTREME.to_string(), price);
    }
    
    fn check_exit(&self, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) -> Option<f32> {
        let mut extreme = *signal_result.position.strategy_attributes.get(KEY_EXTREME).expect("High not set");
        let price_type = if signal_result.signal.action == "buy" {(Tick::Bid, Ohlc::High)} else {(Tick::Ask, Ohlc::Low)};
        let price = prices.last().unwrap().get(&price_type);
        if (signal_result.signal.action == "buy"  && price > extreme) || (signal_result.signal.action == "sell" && price < extreme) {
            extreme = price;
            signal_result.position.strategy_attributes.insert(KEY_EXTREME.to_string(), extreme);
        }

        let sl_factor = self.sl_factor;
        let factor = if signal_result.signal.action == "buy" {1.0 - sl_factor} else {1.0 + sl_factor};
        let stop_loss = extreme * factor;
        signal_result.position.strategy_attributes.insert(KEY_SL.to_string(), stop_loss);

        if signal_result.signal.action == "buy" {
            let price_type_check = (Tick::Bid, Ohlc::Low);
            let price_check = prices.last().unwrap().get(&price_type_check);
            if price_check < signal_result.position.price_open.unwrap() || stop_loss < signal_result.position.price_open.unwrap() {
                return None;
            }
        } else {
            let price_type_check = (Tick::Ask, Ohlc::High);
            let price_check = prices.last().unwrap().get(&price_type_check);
            if price_check > signal_result.position.price_open.unwrap() || stop_loss > signal_result.position.price_open.unwrap() {
                return None;
            }
        }
        
        let price_type_check = if signal_result.signal.action == "buy" {(Tick::Bid, Ohlc::Low)} else {(Tick::Ask, Ohlc::High)};
        let price_check = prices.last().unwrap().get(&price_type_check);
        if (signal_result.signal.action == "buy" && price_check <= stop_loss) || (signal_result.signal.action == "sell" && price_check >= stop_loss) {
            Some(stop_loss)
            } else {
           None
        }
    }
}
use dict_derive::{FromPyObject, IntoPyObject};

use super::price::{Ohlc, Price, PriceType};

#[derive(Clone, Debug, FromPyObject, IntoPyObject)]
pub struct PriceOhlc {
    time_stamp: u64,
    open: f32,
    high: f32,
    low: f32,
    close: f32,
}

impl Price for PriceOhlc {
    fn get(&self, price_type: &PriceType) -> f32 {
        match price_type.1 {
            Ohlc::Open => self.open,
            Ohlc::High => self.high,
            Ohlc::Low => self.low,
            Ohlc::Close => self.close,
        }
    }

    fn ts(&self) -> u64 {
        self.time_stamp
    }
}
use dict_derive::{FromPyObject, IntoPyObject};

use super::price::{Price, PriceType, Tick};


#[derive(Clone, Debug, FromPyObject, IntoPyObject)]
pub struct PriceTick {
    time_stamp: u64,
    ask: f32,
    bid: f32,
}

impl Price for PriceTick {
    fn get(&self, price_type: &PriceType) -> f32 {
        match price_type.0 {
            Tick::Ask => self.ask,
            Tick::Bid => self.bid,
        }
    }

    fn ts(&self) -> u64 {
        self.time_stamp
    }
}

#[derive(Debug, Clone)]
pub enum Tick {
    Ask,
    Bid,
}

#[derive(Debug, Clone)]
pub enum Ohlc {
    Open,
    High,
    Low,
    Close,
}

pub type PriceType = (Tick, Ohlc);

pub trait Price: Send + Sync {
    fn get(&self, price_type: &PriceType) -> f32;
    fn ts(&self) -> u64;
}
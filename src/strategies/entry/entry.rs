use crate::{evaluation::signal::SignalResult, prices::price::Price};

pub trait Entry: Sync + Send {
    fn on_init(&self, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]);
    fn check_entry(&self, signal_result: &SignalResult, prices: &[Box<dyn Price>]) -> Option<f32>;
}
use crate::{evaluation::signal::SignalResult, prices::price::Price};

pub trait Exit: Sync + Send {
    fn on_open(&self, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]);
    fn check_exit(&self, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) -> Option<f32>;
}
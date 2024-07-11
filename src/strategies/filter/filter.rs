use crate::{evaluation::signal::Signal, prices::price::Price};

pub trait Filter: Sync + Send {
    fn check_filter(&self,  signal_result: &Signal, prices: &[Box<dyn Price>]) -> bool;
}
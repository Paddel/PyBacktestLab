use crate::{evaluation::signal::Signal, prices::price::Price};

use super::filter::Filter;

pub struct NoFilter;

impl NoFilter {
    pub fn new() -> NoFilter {
        NoFilter {
        }
    }
}

impl Filter for NoFilter {
    fn check_filter(&self, _signal: &Signal, _prices: &[Box<dyn Price>]) -> bool {
        false
    }
}
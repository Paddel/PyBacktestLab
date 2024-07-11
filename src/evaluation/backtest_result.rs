use dict_derive::IntoPyObject;

use super::position::Position;

#[derive(Debug, IntoPyObject)]
pub struct IgnoredCounts {
    pub missing_margin: u32,
    pub price_gap: u32,
    pub no_entry: u32,
    pub end_of_day: u32,
}

#[derive(Debug, IntoPyObject)]
pub struct BacktestResult {
    profit: f32,
    num_trades: usize,
    sortino_ratio: f32,
    positions: Vec<Position>,
    ignored_counts: IgnoredCounts,
    hit_rate: f32,
    profit_per_day: f32,
}

impl BacktestResult {
    pub fn new(profit: f32, num_trades: usize, sortino_ratio: f32, positions: Vec<Position>, ignored_counts: IgnoredCounts, hit_rate: f32, profit_per_day: f32) -> Self {
        BacktestResult {
            profit,
            num_trades,
            sortino_ratio,
            positions,
            ignored_counts,
            hit_rate,
            profit_per_day,
        }
    }
}
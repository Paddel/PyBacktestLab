use std::sync::RwLock;
use std::vec::Vec;
use std::collections::HashMap;

use chrono::Datelike;
use rayon::prelude::*;

use crate::algorithms::Algorithms;
use crate::prices::price::{Ohlc, Price, Tick};
use crate::prices::price_manager::PriceManager;
use crate::strategies::strategy::{Strategy, StrategyManager, StrategyRules};
use super::backtest_conditions::BacktestConditions;
use super::backtest_result::{BacktestResult, IgnoredCounts};
use super::position::Position;
use super::signal::{Signal, SignalResult};

const MAX_PRICE_DELAY_SECONDS: i128 = 60*15;
const MAX_THREADS: usize = 15;

pub struct BacktestingSignals<'a> {
    price_manager: &'a RwLock<PriceManager>,
}

impl<'a> BacktestingSignals<'a> {
    pub fn new(price_manager: &'a RwLock<PriceManager>) -> BacktestingSignals {
        BacktestingSignals {
            price_manager,
        }
    }

    fn signal_check_init(&self, strategy: &Strategy, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) {
        if strategy.filter.check_filter(&signal_result.signal, prices) {
            return;
        }
        signal_result.position.inited = true;
        strategy.entry.on_init(signal_result, prices);
    }

    fn signal_check_open(&self, strategy: &Strategy, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) {
        let price_open = strategy.entry.check_entry(signal_result, prices);
        if price_open.is_none() {
            return;
        }
        let last_price = prices.last().expect("No prices found");
        signal_result.position.time_stamp_open = Some(last_price.ts());
        signal_result.position.price_open = price_open;
        strategy.exit.on_open(signal_result, prices);

        self.signal_check_close(strategy, signal_result, prices);
        if signal_result.position.closed {
            return;
        }

        signal_result.position.opened = true;
    }

    fn signal_check_close(&self, strategy: &Strategy, signal_result: &mut SignalResult, prices: &[Box<dyn Price>]) {
        let last_price = prices.last().expect("No prices found");
        let price_close = {
            if Algorithms::check_stop_loss_hit(&signal_result.signal, last_price.as_ref()) {
                Some(signal_result.signal.stop_loss)
            } else {
                let price = strategy.exit.check_exit(signal_result, prices);
                if price.is_some() {
                }
                price
            }
        };

        if price_close.is_none() {
            return;
        }
        
        signal_result.position.time_stamp_close = Some(last_price.ts());
        signal_result.position.price_close = price_close;
        
        signal_result.position.closed = true;
    }

    fn is_weekend(timestamp: u64) -> bool {
        let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0).unwrap();
        let weekday = datetime.weekday();
        
        match weekday {
            chrono::Weekday::Sat | chrono::Weekday::Sun => true,
            _ => false,
        }    
    }

    fn backtest(&self, conditions: &BacktestConditions, strategy: &Strategy, mut signals: Vec<Signal>, symbol: String) -> Vec<SignalResult> {
        let mut signals_result: Vec<SignalResult> = Vec::new();
        let mut signals_running: Vec<SignalResult> = Vec::new();
        let price_manager = self.price_manager.read().unwrap();
        let prices: Option<&Vec<Box<dyn Price>>> = price_manager.prices.get(&symbol);
        if prices.is_none() || prices.unwrap().len() == 0 {
            return signals_result;
        }
        let prices = prices.unwrap();
        for p in 0..prices.len() - 1 {
            // Check if signals have to be initiated
            while !signals.is_empty() && prices[p+1].ts() > signals[0].time_stamp {
                let signal = signals.remove(0);
                let mut signal_result = SignalResult::new(signal.clone());
                let delay = (prices[p+1].ts() as i128 - signal.time_stamp as i128) / 1000;
                if delay > MAX_PRICE_DELAY_SECONDS || !Self::is_weekend(signal.time_stamp) {
                    continue;
                }
                self.signal_check_init(strategy, &mut signal_result, &prices[..p+1]);
                if !signal_result.position.inited {
                    continue;
                }
                signals_running.push(signal_result);
            }

            if signals_running.len() == 0 {
                continue;
            }

            // Check if there is a time gap in the prices
            if (prices[p+1].ts() as i128 - prices[p].ts() as i128) / 1000 > MAX_PRICE_DELAY_SECONDS as i128 {
                signals_running.iter_mut().for_each(|result| result.position.ignored.price_gap = true);
                signals_result.extend(signals_running.clone());
                signals_running.clear();
                continue;
            }
            
            // Close signals if this is the last price for this day (22:59)
            if p == prices.len() - 2 || prices[p].ts() / (24*60*60*1000) != (prices[p+1].ts()) / (24*60*60*1000) {
                for signal_result in signals_running.iter_mut() {
                    if signal_result.position.opened {
                        continue;
                    }
                    signal_result.position.ignored.end_of_day = true;
                    signals_result.push(signal_result.clone());
                }
                signals_running.retain(|result: &SignalResult| !result.position.ignored.end_of_day);
                signals_running.iter_mut().for_each(|result| result.position.closed = true);
            }

            
            // Check if not opened signals hit stop loss or are too old and remove them
            for signal_result in signals_running.iter_mut() {
                if signal_result.position.opened || (!Algorithms::check_stop_loss_hit(&signal_result.signal, prices[p].as_ref()) && 
                    prices[p].ts() as i128 - signal_result.signal.time_stamp as i128 <= 20 * 60 * 1000) {
                    continue;
                }
                signal_result.position.ignored.no_entry = true;
                signals_result.push(signal_result.clone());
            }
            signals_running.retain(|result: &SignalResult| !result.position.ignored.no_entry);

            // Check if open signals have to be opened
            for signal_result in signals_running.iter_mut() {
                if signal_result.position.opened {
                    continue;
                }
                self.signal_check_open(strategy, signal_result, &prices[..p+1]);
            }

            // Check for exit conditions
            for signal_result in signals_running.iter_mut() {
                if !signal_result.position.opened {
                    continue;
                }
                self.signal_check_close(strategy, signal_result, &prices[..p+1]);
            }

            // Execute exit for closed signals
            for signal_result in signals_running.iter_mut() {
                if !signal_result.position.closed || !signal_result.position.opened {
                    continue;
                }
                if signal_result.position.price_close == None {
                    let price_type = if signal_result.signal.action == "buy" { (Tick::Bid, Ohlc::Close) } else { (Tick::Ask, Ohlc::Close) };
                    signal_result.position.price_close = Some(prices[p].get(&price_type));
                    signal_result.position.time_stamp_close = Some(prices[p].ts());
                }
                let mut delta = signal_result.position.price_close.expect("Price close not set") - signal_result.position.price_open.expect("Price open not set");
                let action_multiplier = if signal_result.signal.action == "buy" { 1.0 } else { -1.0 };
                let contract_size = *conditions.contract_sizes.get(&symbol).expect("Contract size not found") as f32;
                delta *= contract_size * conditions.lot_size * action_multiplier;
                delta -= conditions.commission * conditions.lot_size * 2.0;
                signal_result.position.delta = Some(delta);
                signals_result.push(signal_result.clone());
            }

            // Remove closed signals
            signals_running.retain(|result| !result.position.closed);

            if signals_running.len() == 0 && signals.len() == 0 {
                break;
            }
        }
        signals_result
    }

    fn group_signals_by_source(&self, signals: Vec<Signal>) -> HashMap<String, Vec<Signal>> {
        let mut signals_by_source: HashMap<String, Vec<Signal>> = HashMap::new();
        for signal in signals {
            let source = signal.source.clone();
            if signals_by_source.contains_key(&source) {
                let signals_for_source = signals_by_source.get_mut(&source).unwrap();
                signals_for_source.push(signal);
            } else {
                signals_by_source.insert(source, vec![signal]);
            }
        }
        signals_by_source
    }

    fn group_signals_by_symbol(&self, signals: Vec<Signal>) -> HashMap<String, Vec<Signal>> {
        let mut signals_by_symbol: HashMap<String, Vec<Signal>> = HashMap::new();
        for signal in signals {
            let symbol = signal.symbol.clone();
            if signals_by_symbol.contains_key(&symbol) {
                let signals_for_symbol = signals_by_symbol.get_mut(&symbol).unwrap();
                signals_for_symbol.push(signal);
            } else {
                signals_by_symbol.insert(symbol, vec![signal]);
            }
        }
        signals_by_symbol
    }

    fn backtest_eval_results(&self, conditions: BacktestConditions, mut results: Vec<SignalResult>) -> BacktestResult {
        results.sort_by_key(|result| result.position.time_stamp_open);

        let mut ignored_counts = IgnoredCounts {
            missing_margin: 0,
            price_gap: 0,
            no_entry: 0,
            end_of_day: 0,
        };
        for result in results.iter_mut() {
            if result.position.ignored.price_gap {
                ignored_counts.price_gap += 1;	
            }
            if result.position.ignored.no_entry {
                ignored_counts.no_entry += 1;
            }
            if result.position.ignored.end_of_day {
                ignored_counts.end_of_day += 1;
            }
        }
        results.retain(|result| result.position.closed);

        let mut rolling_window: Vec<Position> = Vec::new();
        let mut rolling_sum = 0.0;
        let mut profit = 0.0;
        let mut num_trades = 0;
        let mut hit_rate = 0.0;
        let mut positions = Vec::new();

        let mut realized_returns_daily: Vec<f32> = Vec::new();
        let mut realized_returns_window: Vec<Position> = Vec::new();
        let mut negative_deltas: Vec<f32> = Vec::new();
        let risk_free_rate = 0.04 / (21.0 * 12.0) + 1.0;

        for result in results {
            while !rolling_window.is_empty() && result.position.time_stamp_open > rolling_window[0].time_stamp_close {
                let contract_size = *conditions.contract_sizes.get(&result.signal.symbol).expect("Contract size not found") as f32;
                rolling_sum -= rolling_window.remove(0).price_close.expect("Price close not set") * contract_size * conditions.lot_size;
            }

            let contract_size = *conditions.contract_sizes.get(&result.signal.symbol).expect("Contract size not found") as f32;
            let margin = result.position.price_open.expect("Price open not set") * contract_size * conditions.lot_size;
            if rolling_sum + margin < conditions.max_margin {
                rolling_window.push(result.position.clone());
                rolling_sum += margin;
                let delta = result.position.delta.expect("Delta not set");
                profit += delta;
                num_trades += 1;
                hit_rate += if delta > 0.0 { 1.0 } else { -1.0 };

                positions.push(result.position.clone());
                if realized_returns_window.len() > 0 && result.position.time_stamp_close.unwrap() as i128 - realized_returns_window[0].time_stamp_close.unwrap() as i128 > 24*60*60*1000 {
                    let realized_return = realized_returns_window.iter().map(|position| position.delta.expect("Delta not set")).sum::<f32>() / (margin * realized_returns_window.len() as f32) * 1.0;
                    realized_returns_daily.push(realized_return);
                    realized_returns_window.clear();
                    
                    let target_return = realized_return - risk_free_rate;
                    if target_return < 0.0 {
                        negative_deltas.push(target_return * target_return);
                    }
                }
                realized_returns_window.push(result.position.clone());
            }
            else {
                ignored_counts.missing_margin += 1;
            }
        }

        hit_rate /= num_trades as f32;
        let profit_per_day = realized_returns_daily.iter().sum::<f32>() / realized_returns_daily.len() as f32;
        let realized_returns = profit_per_day + 1.0;
        let downside_deviation = (negative_deltas.iter().sum::<f32>() / negative_deltas.len() as f32).sqrt();
        let sortino_ratio = {
            if downside_deviation > 0.0 {
                (realized_returns - risk_free_rate) / downside_deviation
            } else {
                0.0
            }
        };
        BacktestResult::new(profit, num_trades, sortino_ratio, positions, ignored_counts, hit_rate, profit_per_day)
    }

    fn backtest_source(&self, source: &str, conditions: &BacktestConditions, strategy_rules: &HashMap<String, StrategyRules>, signals: Vec<Signal>) -> Vec<SignalResult> {
        let stategy_rules_source = strategy_rules.get(source).expect("Strategy rules not found");
        let strategy = StrategyManager::convert_rules_to_strategy(stategy_rules_source).expect("Failed to convert rules to strategy");
        let mut results: Vec<SignalResult> = Vec::new();
        let signals_by_symbol = self.group_signals_by_symbol(signals.clone());
        for (symbol, signals) in signals_by_symbol.iter() {
            let result = self.backtest(conditions, &strategy, signals.clone(), symbol.clone());
            results.extend(result);
        }
        results
    }

    pub fn backtest_execute(&self, conditions: BacktestConditions, strategy_rules: &HashMap<String, StrategyRules>, signals: Vec<Signal>) -> HashMap<String, BacktestResult> {
        if self.price_manager.read().unwrap().prices.is_empty() {
            println!("WARNING: Prices not set before backtesting. This will result in empty backtest results.");
        }
        
        let signals_by_source = self.group_signals_by_source(signals);
        let pool = rayon::ThreadPoolBuilder::new().num_threads(MAX_THREADS).build().unwrap();

        let result = pool.install(|| {
            signals_by_source.par_iter().map(|(source, signals)| {
                let signal_results = self.backtest_source(source, &conditions, &strategy_rules, signals.clone());
                let backtest_result = self.backtest_eval_results(conditions.clone(), signal_results);
                (source.clone(), backtest_result)
            }).collect::<HashMap<String, BacktestResult>>()
        });
        result
    }

    pub fn check_filter_extern(&self, strategy_rules: StrategyRules, signal: Signal) -> SignalResult {
        let strategy = StrategyManager::convert_rules_to_strategy(&strategy_rules).expect("Failed to convert rules to strategy");
        let price_manager = self.price_manager.read().unwrap();
        let prices = price_manager.prices.get(&signal.symbol).expect("Prices not found");
        let mut signal_result = SignalResult::new(signal);
        self.signal_check_init(&strategy, &mut signal_result, &prices);
        signal_result
    }

    pub fn check_entry_extern(&self, strategy_rules: StrategyRules, mut signal_result: SignalResult) -> SignalResult {
        let strategy = StrategyManager::convert_rules_to_strategy(&strategy_rules).expect("Failed to convert rules to strategy");
        let price_manager = self.price_manager.read().unwrap();
        let prices = price_manager.prices.get(&signal_result.signal.symbol).expect("Prices not found");
        self.signal_check_open(&strategy, &mut signal_result, &prices);
        signal_result
    }

    pub fn check_exit_extern(&self, strategy_rules: StrategyRules, mut signal_result: SignalResult) -> SignalResult {
        let strategy = StrategyManager::convert_rules_to_strategy(&strategy_rules).expect("Failed to convert rules to strategy");
        let price_manager = self.price_manager.read().unwrap();
        let prices = price_manager.prices.get(&signal_result.signal.symbol).expect("Prices not found");
        self.signal_check_close(&strategy, &mut signal_result, &prices);
        signal_result
    }
}
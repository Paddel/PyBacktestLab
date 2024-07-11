use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use algorithms::Algorithms;
use once_cell::sync::Lazy;
use prices::price_manager::PriceManager;
use prices::price_ohlc::PriceOhlc;
use prices::price_tick::PriceTick;
use prices::price::PriceType;
use pyo3::prelude::*;

use evaluation::backtest_conditions::BacktestConditions;
use evaluation::backtest_result::BacktestResult;
use evaluation::backtesting_signals::BacktestingSignals;
use evaluation::signal::{Signal, SignalResult};
use strategies::strategy::StrategyRules;
use prices::price::{Ohlc, Price, Tick};

mod algorithms;
mod evaluation;
mod strategies;
mod prices;

static PRICE_MANAGER: Lazy<RwLock<PriceManager>> = Lazy::new(|| RwLock::new(PriceManager::new()));
static BACKTESTING_SIGNALS: Lazy<Mutex<BacktestingSignals>> = Lazy::new(|| Mutex::new(BacktestingSignals::new(&PRICE_MANAGER)));

fn convert_price_type(type_tick: &str, type_ohlc: &str) -> PriceType {
    match type_tick {
        "ask" => {
            match type_ohlc {
                "open" => (Tick::Ask, Ohlc::Open),
                "high" => (Tick::Ask, Ohlc::High),
                "low" => (Tick::Ask, Ohlc::Low),
                "close" => (Tick::Ask, Ohlc::Close),
                _ => panic!("Invalid ohlc type")
            }
        },
        "bid" => {
            match type_ohlc {
                "open" => (Tick::Bid, Ohlc::Open),
                "high" => (Tick::Bid, Ohlc::High),
                "low" => (Tick::Bid, Ohlc::Low),
                "close" => (Tick::Bid, Ohlc::Close),
                _ => panic!("Invalid ohlc type")
            }
        },
        _ => panic!("Invalid tick type")
    }
}

#[pyfunction]
fn prices_tick_add(prices: HashMap<String, Vec<PriceTick>>) {
    let mut price_manager = PRICE_MANAGER.write().expect("Failed to lock backtesting for 'prices_add'");
    let mut converted_prices: HashMap<String, Vec<Box<dyn Price>>> = HashMap::new();
    for (key, value) in prices {
        let converted_value: Vec<Box<dyn Price>> = value.into_iter().map(|py_price_tick| Box::new(PriceTick::from(py_price_tick)) as Box<dyn Price>).collect();
        converted_prices.insert(key, converted_value);
    }
    price_manager.add_prices(converted_prices);
}

#[pyfunction]
fn prices_ohlc_add(prices: HashMap<String, Vec<PriceOhlc>>) {
    let mut price_manager = PRICE_MANAGER.write().expect("Failed to lock backtesting for 'prices_add'");
    let mut converted_prices: HashMap<String, Vec<Box<dyn Price>>> = HashMap::new();
    for (key, value) in prices {
        let converted_value: Vec<Box<dyn Price>> = value.into_iter().map(|v| Box::new(v) as Box<dyn Price>).collect();
        converted_prices.insert(key, converted_value);
    }
    price_manager.add_prices(converted_prices);
}

#[pyfunction]
fn backtest_signals(conditions: BacktestConditions, strategy_rules: HashMap<String, StrategyRules>, signals: Vec<Signal>) -> PyResult<HashMap<String, BacktestResult>> {
    let backtesting_signals = BACKTESTING_SIGNALS.lock().expect("Failed to lock backtesting for 'backtest'");
    Ok(backtesting_signals.backtest_execute(conditions, &strategy_rules, signals))
}

#[pyfunction]
fn signal_check_filter(strategy_rules: StrategyRules, signal: Signal) -> PyResult<SignalResult> {
    let backtesting_signals = BACKTESTING_SIGNALS.lock().expect("Failed to lock backtesting for 'signal_check_filter'");
    Ok(backtesting_signals.check_filter_extern(strategy_rules, signal))
}

#[pyfunction]
fn signal_check_entry(strategy_rules: StrategyRules, signal_result: SignalResult) -> PyResult<SignalResult> {
    let backtesting_signals = BACKTESTING_SIGNALS.lock().expect("Failed to lock backtesting for 'signal_check_entry'");
    Ok(backtesting_signals.check_entry_extern(strategy_rules, signal_result))
}

#[pyfunction]
fn signal_check_exit(strategy_rules: StrategyRules, signal_result: SignalResult) -> PyResult<SignalResult> {
    let backtesting_signals = BACKTESTING_SIGNALS.lock().expect("Failed to lock backtesting for 'signal_check_exit'");
    Ok(backtesting_signals.check_exit_extern(strategy_rules, signal_result))
}

#[pyfunction]
fn algo_mean(symbol: &str, type_tick: &str, type_ohlc: &str, index_from: usize, index_to: usize, minutes: f32) -> f32 {
    let price_manager = PRICE_MANAGER.read().expect("Failed to lock backtesting for 'algo_mean'");
    let prices = price_manager.prices.get(symbol).expect("Failed to get prices for 'algo_mean'");
    let price_type = convert_price_type(type_tick, type_ohlc);
    assert!(index_from < prices.len(), "Index from is out of bounds for 'algo_mean'");
    assert!(index_to < prices.len(), "Index to is out of bounds for 'algo_mean'");
    return Algorithms::calculate_mean(&price_type, &prices[index_from..index_to+1], minutes);
}

#[pyfunction]
fn algo_std_dev(symbol: &str, type_tick: &str, type_ohlc: &str, index_from: usize, index_to: usize, minutes: f32, mean: f32) -> f32 {
    let price_manager = PRICE_MANAGER.read().expect("Failed to lock backtesting for 'algo_std_dev'");
    let prices = price_manager.prices.get(symbol).expect("Failed to get prices for 'algo_std_dev'");
    let price_type = convert_price_type(type_tick, type_ohlc);
    assert!(index_from < prices.len(), "Index from is out of bounds for 'algo_std_dev'");
    assert!(index_to < prices.len(), "Index to is out of bounds for 'algo_std_dev'");
    return Algorithms::calculate_standard_deviation(&price_type, &prices[index_from..index_to+1], minutes, mean);
}

#[pymodule]
fn py_backtest_lab(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(prices_tick_add, m)?)?;
    m.add_function(wrap_pyfunction!(prices_ohlc_add, m)?)?;
    m.add_function(wrap_pyfunction!(backtest_signals, m)?)?;

    m.add_function(wrap_pyfunction!(signal_check_filter, m)?)?;
    m.add_function(wrap_pyfunction!(signal_check_entry, m)?)?;
    m.add_function(wrap_pyfunction!(signal_check_exit, m)?)?;

    m.add_function(wrap_pyfunction!(algo_mean, m)?)?;
    m.add_function(wrap_pyfunction!(algo_std_dev, m)?)?;
    Ok(())
}

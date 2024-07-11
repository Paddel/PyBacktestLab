use std::{collections::HashMap, sync::RwLock};

use once_cell::sync::Lazy;

use prices::{price::Price, price_manager::PriceManager, price_ohlc::PriceOhlc, price_tick::PriceTick};
use pyo3::prelude::*;

mod prices;

static PRICE_MANAGER: Lazy<RwLock<PriceManager>> = Lazy::new(|| RwLock::new(PriceManager::new()));

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

#[pymodule]
fn py_backtest_lab(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(prices_tick_add, m)?)?;
    m.add_function(wrap_pyfunction!(prices_ohlc_add, m)?)?;
    Ok(())
}

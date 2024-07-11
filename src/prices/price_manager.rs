use std::collections::HashMap;

use super::price::Price;

pub struct PriceManager {
    pub prices: HashMap<String, Vec<Box<dyn Price>>>,
}

impl PriceManager {
    pub fn new() -> Self {
        Self {
            prices: HashMap::new(),
        }
    }

    pub fn add_prices(&mut self, prices: HashMap<String, Vec<Box<dyn Price>>>) {
        for (key, value) in prices {
            self.prices.entry(key).or_insert_with(Vec::new).extend(value);
        }
    }
}
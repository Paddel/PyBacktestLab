use crate::{evaluation::signal::Signal, prices::price::{Ohlc, Price, PriceType, Tick}};

pub struct Algorithms;

impl Algorithms {
    pub fn check_stop_loss_hit(signal: &Signal, price: &dyn Price) -> bool {
        if signal.action == "buy" && price.get(&(Tick::Bid, Ohlc::Low)) <= signal.stop_loss {
            return true;
        } else if signal.action == "sell" && price.get(&(Tick::Ask, Ohlc::High)) >= signal.stop_loss {
            return true;
        }
        false
    }

    pub fn calculate_volatility(price_type: &PriceType, prices: &[Box<dyn Price>], period_minutes: f32) -> f32 {
        let mut windows = Vec::new();
        let mut rolling_window = Vec::new();
        let time_stamp_first = prices.last().unwrap().ts();
        let mut last_insert = time_stamp_first;
        
        for price in prices.iter().rev() {
            if (time_stamp_first as i128 - price.ts() as i128) / 1000 / 60 > period_minutes as i128{
                break;
            }
            
            rolling_window.push(price.get(&price_type));
            
            if (last_insert as i128 - price.ts() as i128) / 1000 / 60 > 5 {
                windows.push(rolling_window.clone());
                rolling_window.clear();
                last_insert = price.ts();
            }
        }

        let mut windows_avg = 0.0;
        if !windows.is_empty() {
            for window in &windows {
                let min_price = window.iter().cloned().fold(f32::NAN, f32::min);
                let max_price = window.iter().cloned().fold(f32::NAN, f32::max);
                windows_avg += (max_price - min_price) / min_price;
            }
            windows_avg /= windows.len() as f32;
        }
        
        windows_avg
    }

    pub fn calculate_volatility_factor(action: &str, volatiliy: f32, strategy_factor: f32) -> f32 {
        let action_factor = if action == "buy" {1.0} else {-1.0};
        1.0 + action_factor * strategy_factor * volatiliy
    }

    pub fn calculate_mean(price_type: &PriceType, prices: &[Box<dyn Price>], minutes: f32) -> f32 {
        let mut rolling_window = Vec::new();
        let time_stamp_first = prices.last().unwrap().ts();
        
        for price in prices.iter().rev() {
            if (time_stamp_first as i128 - price.ts() as i128) / 1000 / 60 > minutes as i128{
                break;
            }
            rolling_window.push(price.get(&price_type));
        }

        rolling_window.iter().sum::<f32>() / rolling_window.len() as f32
    }

    pub fn calculate_standard_deviation(price_type: &PriceType, prices: &[Box<dyn Price>], minutes: f32, mean: f32) -> f32 {
        let mut deviation_sum = 0.0;
        let time_stamp_first = prices.last().unwrap().ts();
        let mut num_prices = 0;
    
        for price in prices.iter().rev() {
            if (time_stamp_first as i128 - price.ts() as i128) / 1000 / 60 > minutes as i128{
                break;
            }
            let price_value = price.get(&price_type);
            deviation_sum += (price_value - mean).powi(2);
            num_prices += 1;
        }
    
        let variance = deviation_sum / (num_prices as f32); 
        variance.sqrt()
    }
}
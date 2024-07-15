use std::collections::HashMap;

use dict_derive::FromPyObject;

use super::entry::entry::Entry;
use super::entry::bollinger::Bollinger as BollingerEntry;
use super::entry::immediate::Immediate;
use super::entry::volatility_mean::VolatilityMean;
use super::entry::volatility_pullback::VolatilityPullback;
use super::exit::exit::Exit;
use super::exit::bollinger::Bollinger as BollingerExit;
use super::exit::fixed_tp::FixedTP;
use super::exit::trailing_stop::TrailingStop;
use super::filter::filter::Filter;
use super::filter::no_filter::NoFilter;

#[derive(Clone, Debug, FromPyObject)]
struct Rule {
    name: String,
    parameters: HashMap<String, f32>,
}

#[derive(Clone, Debug, FromPyObject)]
pub struct StrategyRules {
    entry: Rule,
    exit: Rule,
    filter: Rule,
}

pub struct Strategy {
    pub entry: Box<dyn Entry>,
    pub exit: Box<dyn Exit>,
    pub filter: Box<dyn Filter>,
}

pub struct StrategyManager {
}

impl StrategyManager {
    fn map_strategy_entry(rule: &Rule) -> Result<Box<dyn Entry>, &'static str> {
        match rule.name.as_str() {
            "volatility_mean" => {
                let entry_factor = rule.parameters.get("entry_factor").ok_or("Missing entry_factor")?;
                let vol_timeframe = rule.parameters.get("vol_timeframe").ok_or("Missing vol_timeframe")?;
                Ok(Box::new(VolatilityMean::new(*entry_factor, *vol_timeframe)))
            }
            "volatility_pullback" => {
                let entry_factor = rule.parameters.get("entry_factor").ok_or("Missing entry_factor")?;
                let vol_timeframe = rule.parameters.get("vol_timeframe").ok_or("Missing vol_timeframe")?;
                Ok(Box::new(VolatilityPullback::new(*entry_factor, *vol_timeframe)))
            }
            "bollinger" => {
                let std_dev_factor = rule.parameters.get("std_dev_factor").ok_or("Missing std_dev_factor")?;
                let period_minutes = rule.parameters.get("period_minutes").ok_or("Missing period_minutes")?;
                Ok(Box::new(BollingerEntry::new(*std_dev_factor, *period_minutes as i32)))
            }
            "immediate" => Ok(Box::new(Immediate::new())),
            _ => Err("Invalid entry strategy name"),
        }
    }

    fn map_strategy_exit(rule: &Rule) -> Result<Box<dyn Exit>, &'static str> {
        match rule.name.as_str() {
            "bollinger" => {
                let std_dev_factor = rule.parameters.get("std_dev_factor").ok_or("Missing std_dev_factor")?;
                let period_minutes = rule.parameters.get("period_minutes").ok_or("Missing period_minutes")?;
                Ok(Box::new(BollingerExit::new(*std_dev_factor, *period_minutes as i32)))
            }
            "fixed_tp" => {
                let tp_factor = rule.parameters.get("tp_factor").ok_or("Missing tp_factor")?;
                let vol_timeframe = rule.parameters.get("vol_timeframe").ok_or("Missing vol_timeframe")?;
                Ok(Box::new(FixedTP::new(*tp_factor, *vol_timeframe)))
            }
            "trailing_stop" => {
                let sl_factor = rule.parameters.get("sl_factor").ok_or("Missing sl_factor")?;
                Ok(Box::new(TrailingStop::new(*sl_factor)))
            }
            _ => Err("Invalid exit strategy name"),
        }
    }

    fn map_strategy_filter(rule: &Rule) -> Result<Box<dyn Filter>, &'static str> {
        match rule.name.as_str() {
            "no_filter" => Ok(Box::new(NoFilter::new())),
            _ => Err("Invalid filter strategy name"),
        }
    }

    pub fn convert_rules_to_strategy(rules: &StrategyRules) -> Result<Strategy, &'static str> {
        let entry = StrategyManager::map_strategy_entry(&rules.entry)?;
        let exit = StrategyManager::map_strategy_exit(&rules.exit)?;
        let filter = StrategyManager::map_strategy_filter(&rules.filter)?;

        Ok(Strategy {
            entry,
            exit,
            filter,
        })
    }
}
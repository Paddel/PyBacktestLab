use std::collections::HashMap;

use dict_derive::{FromPyObject, IntoPyObject};

#[derive(Clone, Debug, FromPyObject, IntoPyObject)]
pub struct PositionIgnored {
    pub price_gap: bool,
    pub no_entry: bool,
    pub end_of_day: bool,
}

impl PositionIgnored {
    pub fn new() -> Self {
        PositionIgnored {
            price_gap: false,
            no_entry: false,
            end_of_day: false,
        }
    }
}

#[derive(Clone, Debug, FromPyObject, IntoPyObject)]
pub struct Position {
    pub action: String,
    pub ignored: PositionIgnored,
    pub time_stamp_open: Option<u64>,
    pub time_stamp_close: Option<u64>,
    pub price_open: Option<f32>,
    pub price_close: Option<f32>,
    pub delta: Option<f32>,
    pub inited: bool,
    pub opened: bool,
    pub closed: bool,
    pub strategy_attributes: HashMap<String, f32>,
}

impl Position {
    pub fn new(action: &str) -> Self {
        Position {
            action: action.to_string(),
            ignored: PositionIgnored::new(),
            time_stamp_open: None,
            time_stamp_close: None,
            price_open: None,
            price_close: None,
            delta: None,
            inited: false,
            opened: false,
            closed: false,
            strategy_attributes: HashMap::new(),
        }
    }
}
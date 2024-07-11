use std::collections::HashMap;

use dict_derive::FromPyObject;

#[derive(Clone, Debug, FromPyObject)]
pub struct BacktestConditions {
    pub max_margin: f32,
    pub commission: f32,
    pub lot_size: f32,
    pub contract_sizes: HashMap<String, u32>,
}
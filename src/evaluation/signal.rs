use dict_derive::{FromPyObject, IntoPyObject};

use super::position::Position;

#[derive(Clone, Debug, FromPyObject, IntoPyObject)]
pub struct Signal {
    pub symbol: String,
    pub action: String,
    pub stop_loss: f32,
    pub take_profit: Vec<f32>,
    pub time_stamp: u64,
    pub source: String,
}

#[derive(Clone, Debug, FromPyObject, IntoPyObject)]
pub struct SignalResult {
    pub signal: Signal,
    pub position: Position,
    pub ticket: Option<u32>,
}

impl SignalResult {
    pub fn new(signal: Signal) -> SignalResult {
        SignalResult {
            position: Position::new(&signal.action),
            signal,
            ticket: None,
        }
    }
}
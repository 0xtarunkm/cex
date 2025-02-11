use serde::{Deserialize, Serialize};

mod message_from_engine;
mod message_to_engine;

pub use message_from_engine::*;
pub use message_to_engine::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderType {
    MarginLong,
    MarginShort,
    Spot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

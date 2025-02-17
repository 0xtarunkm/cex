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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnRampPayload {
    pub user_id: String,
    pub network: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct OpenOrdersQuery {
    pub user_id: String,
    pub market: String,
}

#[derive(Deserialize)]
pub struct GetMarginPositionsQuery {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct GetUserBalancesQuery {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct GetDepthQuery {
    pub market: String,
    pub order_type: OrderType,
}

#[derive(Deserialize)]
pub struct GetTickerQuery {
    pub market: String,
    pub order_type: OrderType,
}

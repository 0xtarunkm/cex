mod incoming_message;
mod message_from_api;
mod message_to_api;
mod order;
mod user;

pub use incoming_message::*;
pub use message_from_api::*;
pub use message_to_api::*;
pub use user::*;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum OrderType {
    MarginLong,
    MarginShort,
    Spot,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Balance {
    pub ticker: String,
    pub balance: Decimal,
    pub locked_balance: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PositionType {
    Long,
    Short,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarginPosition {
    pub asset: String,
    pub user_id: String,
    pub position_type: PositionType,
    pub entry_price: Decimal,
    pub size: Decimal,
    pub leverage: Decimal,
    pub collateral: Decimal,
    pub unrealized_pnl: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub is_margin: bool,
    pub leverage: Option<Decimal>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Depth {
    pub orders: HashMap<Decimal, OrderDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderDetails {
    pub type_: OrderSide,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetQuoteResponse {
    pub avg_price: Decimal,
    pub quantity: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserBalancesPayload {
    pub balances: Vec<Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginPositionsPayload {
    pub positions: Vec<MarginPosition>,
}

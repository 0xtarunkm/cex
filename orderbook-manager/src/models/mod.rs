mod incoming_message;
mod message_from_api;
mod message_to_api;
mod order;
mod user;

pub use incoming_message::*;
pub use message_from_api::*;
pub use message_to_api::*;
pub use order::*;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginPosition {
    pub asset: String,
    pub quantity: Decimal,
    pub avg_price: Decimal,
    pub position_type: OrderType,
    pub unrealized_pnl: Option<Decimal>,
    pub liquidation_price: Option<Decimal>,
}

impl MarginPosition {
    pub fn calculate_unrealized_pnl(&mut self, current_price: Decimal) {
        self.unrealized_pnl = Some(match self.position_type {
            OrderType::MarginLong => (current_price - self.avg_price) * self.quantity,
            OrderType::MarginShort => (self.avg_price - current_price) * self.quantity,
            _ => Decimal::ZERO,
        });
    }

    pub fn calculate_liquidation_price(&mut self, leverage: Decimal, maintenance_margin: Decimal) {
        self.liquidation_price = Some(match self.position_type {
            OrderType::MarginLong => {
                self.avg_price * (Decimal::ONE - maintenance_margin * leverage)
            }
            OrderType::MarginShort => {
                self.avg_price * (Decimal::ONE + maintenance_margin * leverage)
            }
            _ => Decimal::ZERO,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotOrder {
    pub id: String,
    pub user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
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

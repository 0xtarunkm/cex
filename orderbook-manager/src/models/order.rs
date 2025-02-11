use std::collections::HashMap;

use super::{OrderSide, OrderType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginOrder {
    pub id: String,
    pub user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub leverage: Decimal,
    pub order_type: OrderType,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpotOrder {
    pub id: String,
    pub user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderDetails {
    pub type_: OrderSide,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Depth {
    pub orders: HashMap<Decimal, OrderDetails>,
}

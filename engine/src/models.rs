use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub price: Decimal,
    pub quantity: Decimal,
    pub order_id: String,
    pub filled: Decimal,
    pub side: Side,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub bids: BTreeMap<Decimal, Vec<Order>>,
    pub asks: BTreeMap<Decimal, Vec<Order>>,
    pub base_asset: String,
    pub quote_asset: String,
    pub last_trade_id: u64,
    pub current_price: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub price: Decimal,
    pub qty: Decimal,
    pub trade_id: u64,
    pub other_user_id: String,
    pub marker_order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub available: Decimal,
    pub locked: Decimal,
}

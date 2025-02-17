use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MessageFromEngine {
    OrderPlaced {
        payload: OrderPlacedPayload,
    },
    #[serde(rename = "ORDER_CANCELLED")]
    OrderCancelled {
        payload: OrderCancelledPayload,
    },
    #[serde(rename = "OPEN_ORDERS")]
    OpenOrders {
        payload: OpenOrdersPayload,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderPlacedPayload {
    pub order_id: String,
    pub remaining_qty: Decimal,
    pub filled_qty: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderCancelledPayload {
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenOrdersPayload {
    pub open_orders: Vec<SpotOrder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderPayload {
    pub user_id: String,
    pub market: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub leverage: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderType {
    MarginLong,
    MarginShort,
    Spot,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
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

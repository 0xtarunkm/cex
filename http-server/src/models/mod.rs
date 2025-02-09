use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum MessageToEngine {
    #[serde(rename = "CREATE_ORDER")]
    CreateOrder { data: CreateOrderPayload },
    #[serde(rename = "CANCEL_ORDER")]
    CancelOrder { data: CancelOrderPayload },
    #[serde(rename = "GET_DEPTH")]
    GetDepth { data: GetDepthPayload },
    #[serde(rename = "GET_OPEN_ORDERS")]
    GetOpenOrders { data: GetOpenOrdersPayload },
    #[serde(rename = "GET_QUOTE")]
    GetQuote { data: GetQuoteRequest },
    #[serde(rename = "GET_USER_BALANCES")]
    GetUserBalances { data: GetUserBalancesPayload },
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderPayload {
    pub order_id: String,
    pub user_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnRampData {
    pub amount: String,
    pub user_id: String,
    pub txn_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDepthPayload {
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOpenOrdersPayload {
    pub user_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetQuoteRequest {
    pub market: String,
    pub side: OrderSide,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserBalancesPayload {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MessageFromEngine {
    #[serde(rename = "DEPTH")]
    Depth { payload: DepthPayload },
    #[serde(rename = "ORDER_PLACED")]
    OrderPlaced { payload: OrderPlacedPayload },
    #[serde(rename = "ORDER_CANCELLED")]
    OrderCancelled { payload: OrderCancelledPayload },
    #[serde(rename = "OPEN_ORDERS")]
    OpenOrders { payload: OpenOrdersPayload },
    #[serde(rename = "USER_BALANCES")]
    UserBalances { payload: UserBalancesPayload },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub order_type: OrderType,
    pub leverage: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenOrdersPayload {
    pub open_orders: Vec<Order>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderDetails {
    pub type_: OrderSide,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DepthPayload {
    pub orders: HashMap<Decimal, OrderDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderPlacedPayload {
    pub order_id: String,
    pub executed_qty: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fill {
    pub price: Decimal,
    pub quantity: Decimal,
    pub trade_id: u64,
    pub other_user_id: String,
    pub marker_order_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderCancelledPayload {
    pub order_id: String,
    pub executed_qty: Decimal,
    pub remaining_qty: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenOrder {
    pub order_id: String,
    pub executed_qty: f64,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Balance {
    pub ticker: String,
    pub balance: Decimal,
    pub locked_balance: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserBalancesPayload {
    pub balances: Vec<Balance>,
}


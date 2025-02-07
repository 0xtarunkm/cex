use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    pub client_id: String,
    pub message: MessageFromApi,
}

// ORDERS
#[derive(Debug, Clone, Serialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
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
pub struct CreateOrderPayload {
    pub user_id: String,
    pub market: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub leverage: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderDetails {
    pub type_: OrderSide,
    pub quantity: Decimal,
}

// DEPTH
#[derive(Debug, Serialize, Deserialize)]
pub struct Depth {
    pub orders: HashMap<Decimal, OrderDetails>,
    pub market: String,
}

// USER
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub balances: Vec<Balances>,
    pub margin_positions: Vec<MarginPosition>,
    pub margin_enabled: bool,
    pub margin_used: Decimal,
    pub max_leverage: Decimal,
    pub realized_pnl: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balances {
    pub ticker: String,
    pub balance: Decimal,
    pub locked_balance: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarginPosition {
    pub ticker: String,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub liquidation_price: Decimal,
    pub leverage: Decimal,
    pub unrealized_pnl: Decimal,
    pub side: MarginSide,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum MarginSide {
    Long,
    Short,
}

#[derive(Debug, Serialize)]
pub struct Fill {
    pub price: Decimal,
    pub quantity: Decimal,
    pub trade_id: u64,
    pub other_user_id: String,
    pub marker_order_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageFromApi {
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderData {
    pub user_id: String,
    pub market: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub leverage: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderPayload {
    pub order_id: String,
    pub user_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnRampData {
    pub amount: Decimal,
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
pub struct GetQuoteResponse {
    pub avg_price: Decimal,
    pub quantity: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum MessageToApi {
    #[serde(rename = "ORDER_PLACED")]
    OrderPlaced { payload: OrderPlacedPayload },
    #[serde(rename = "ORDER_CANCELLED")]
    OrderCancelled { payload: OrderCancelledPayload },
    #[serde(rename = "OPEN_ORDERS")]
    OpenOrders { payload: OpenOrdersPayload },
    #[serde(rename = "DEPTH")]
    Depth { payload: Depth },
    #[serde(rename = "SEND_QUOTE")]
    Quote { payload: GetQuoteResponse },
}

#[derive(Debug, Serialize)]
pub struct OrderPlacedPayload {
    pub order_id: String,
    pub executed_qty: Decimal,
}

pub enum StatusCode {
    OK,
    NotFound,
}

#[derive(Debug, Serialize)]
pub struct OrderCancelledPayload {
    pub order_id: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenOrdersPayload {
    pub open_orders: Vec<Order>,
}

#[derive(Debug, Serialize)]
pub struct DepthPayload {
    pub bids: Vec<(Decimal, Decimal)>,
    pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageTier {
    pub initial_margin: Decimal,
    pub maintenance_margin: Decimal,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct MarginPosition {
//     pub asset: String,
//     pub size: Decimal,
//     pub entry_price: Decimal,
//     pub leverage: Decimal,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub avg_price: Decimal,
    pub quantity: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum MessageToEngine {
    #[serde(rename = "CREATE_ORDER")]
    CreateOrder { data: CreateOrderData },
    #[serde(rename = "CANCEL_ORDER")]
    CancelOrder { data: CancelOrderData },
    #[serde(rename = "ON_RAMP")]
    OnRamp { data: OnRampData },
    #[serde(rename = "GET_DEPTH")]
    GetDepth { data: GetDepthData },
    #[serde(rename = "GET_OPEN_ORDERS")]
    GetOpenOrders { data: GetOpenOrdersData },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderData {
    pub market: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderData {
    pub order_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnRampData {
    pub amount: String,
    pub user_id: String,
    pub txn_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDepthData {
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOpenOrdersData {
    pub user_id: String,
    pub market: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled: Decimal,
    pub side: OrderSide,
    pub user_id: String,
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenOrdersPayload {
    pub open_orders: Vec<Order>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepthPayload {
    pub bids: Vec<(Decimal, Decimal)>,
    pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderPlacedPayload {
    pub order_id: String,
    pub executed_qty: Decimal,
    pub fills: Vec<Fill>,
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

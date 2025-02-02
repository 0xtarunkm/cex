use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    pub client_id: String,
    pub message: MessageFromApi,
}

#[derive(Debug, Clone, Serialize)]
pub struct Order {
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled: Decimal,
    pub side: OrderSide,
    pub user_id: String,
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize)]
pub struct Fill {
    pub price: Decimal,
    pub quantity: Decimal,
    pub trade_id: u64,
    pub other_user_id: String,
    pub marker_order_id: String,
}

#[derive(Debug)]
pub struct MatchResult {
    pub executed_qty: Decimal,
    pub fills: Vec<Fill>,
}

#[derive(Debug)]
pub struct AssetBalance {
    pub available: Decimal,
    pub locked: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageFromApi {
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
    pub amount: Decimal,
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
    Depth { payload: DepthPayload },
}

#[derive(Debug, Serialize)]
pub struct OrderPlacedPayload {
    pub order_id: String,
    pub executed_qty: Decimal,
    pub fills: Vec<Fill>,
}

#[derive(Debug, Serialize)]
pub struct OrderCancelledPayload {
    pub order_id: String,
    pub executed_qty: Decimal,
    pub remaining_qty: Decimal,
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

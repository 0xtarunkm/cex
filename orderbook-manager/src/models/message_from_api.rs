use super::{OrderSide, OrderType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
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
    #[serde(rename = "GET_USER_BALANCES")]
    GetUserBalances { data: GetUserBalancesPayload },
    #[serde(rename = "GET_MARGIN_POSITIONS")]
    GetMarginPositions { data: GetMarginPositionsPayload },
    #[serde(rename = "GET_TICKER")]
    GetTicker { market: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderPayload {
    pub user_id: String,
    pub market: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub is_margin: bool,
    pub leverage: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderPayload {
    pub order_id: String,
    pub user_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDepthPayload {
    pub market: String,
    pub order_type: OrderType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOpenOrdersPayload {
    pub user_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetQuoteRequest {
    pub market: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserBalancesPayload {
    pub user_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetMarginPositionsPayload {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClosePositionPayload {
    pub user_id: String,
    pub market: String,
    pub position_type: OrderType,
}

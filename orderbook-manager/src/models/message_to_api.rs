use crate::services::price_service::PriceInfo;

use super::{Depth, GetQuoteResponse, MarginPositionsPayload, SpotOrder, UserBalancesPayload};
use rust_decimal::Decimal;
use serde::Serialize;

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
    #[serde(rename = "USER_BALANCES")]
    UserBalances { payload: UserBalancesPayload },
    #[serde(rename = "GET_MARGIN_POSITIONS")]
    GetMarginPositions { payload: MarginPositionsPayload },
    #[serde(rename = "TICKER_PRICE")]
    TickerPrice { 
        market: String,
        price: Option<PriceInfo> 
    },
}

#[derive(Debug, Serialize)]
pub struct OrderPlacedPayload {
    pub order_id: String,
    pub remaining_qty: Decimal,
    pub filled_qty: Decimal,
}

#[derive(Debug, Serialize)]
pub struct OrderCancelledPayload {
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenOrdersPayload {
    pub open_orders: Vec<SpotOrder>,
}


use super::{OrderSide, OrderType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(rename = "GET_MARGIN_POSITIONS")]
    GetMarginPositions { payload: MarginPositionsPayload },
    #[serde(rename = "SEND_QUOTE")]
    SendQuote { payload: GetQuoteResponse },
    #[serde(rename = "TICKER_PRICE")]
    TickerPrice {
        market: String,
        price: Option<PriceInfo>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginPositionsPayload {
    pub positions: Vec<MarginPosition>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenOrdersPayload {
    pub open_orders: Vec<SpotOrder>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotOrder {
    pub id: String,
    pub user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub timestamp: i64,
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
    pub remaining_qty: Decimal,
    pub filled_qty: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderCancelledPayload {
    pub message: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetQuoteResponse {
    pub avg_price: Decimal,
    pub quantity: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceInfo {
    pub last_trade_price: Option<Decimal>,
    pub mark_price: Decimal,
    pub index_price: Option<Decimal>,
    pub timestamp: i64,
}

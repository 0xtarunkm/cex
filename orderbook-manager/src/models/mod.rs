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
pub struct SpotOrder {
    pub id: String,
    pub user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: OrderSide,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum OrderType {
    MarginLong,
    MarginShort,
    Spot,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

// MARGIN ORDERS
#[derive(Debug, Clone)]
pub struct MarginOrder {
    pub id: String,
    pub user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub leverage: Decimal,
    pub order_type: OrderType,
    pub timestamp: i64,
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

impl MarginPosition {
    pub fn calculate_unrealized_pnl(&mut self, current_price: Decimal) {
        self.unrealized_pnl = match self.position_type {
            OrderType::MarginLong => Some((current_price - self.avg_price) * self.quantity),
            OrderType::MarginShort => Some((self.avg_price - current_price) * self.quantity),
            _ => None,
        }
    }

    pub fn calculate_liquidation_price(&mut self, leverage: Decimal, maintenance_margin: Decimal) {
        let maintenance_margin_ratio = maintenance_margin / leverage;

        self.liquidation_price = match self.position_type {
            OrderType::MarginLong => {
                Some(self.avg_price * (Decimal::ONE - maintenance_margin_ratio))
            }
            OrderType::MarginShort => {
                Some(self.avg_price * (Decimal::ONE + maintenance_margin_ratio))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MarginPositionsPayload {
    pub positions: Vec<MarginPosition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarginOrderPlacedPayload {
    pub order_id: String,
    pub remaining_qty: Decimal,
    pub filled_qty: Decimal,
    pub leverage: Decimal,
    pub position_type: OrderType,
}

// DEPTH
#[derive(Debug, Serialize, Deserialize)]
pub struct Depth {
    pub orders: HashMap<Decimal, OrderDetails>,
}

// USER
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub balances: Vec<Balances>,
    pub margin_enabled: bool,
    pub margin_positions: Vec<MarginPosition>,
    pub margin_used: Decimal,
    pub max_leverage: Decimal,
    pub realized_pnl: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Balances {
    pub ticker: String,
    pub balance: Decimal,
    pub locked_balance: Decimal,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct MarginPosition {
//     pub ticker: String,
//     pub size: Decimal,
//     pub entry_price: Decimal,
//     pub liquidation_price: Decimal,
//     pub leverage: Decimal,
//     pub unrealized_pnl: Decimal,
//     pub side: MarginSide,
// }

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
    #[serde(rename = "GET_USER_BALANCES")]
    GetUserBalances { data: GetUserBalancesPayload },
    #[serde(rename = "GET_MARGIN_POSITIONS")]
    GetMarginPositions { data: GetMarginPositionsPayload },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetMarginPositionsPayload {
    pub user_id: String,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetQuoteResponse {
    pub avg_price: Decimal,
    pub quantity: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserBalancesPayload {
    pub user_id: String,
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
    #[serde(rename = "USER_BALANCES")]
    UserBalances { payload: UserBalancesPayload },
    #[serde(rename = "GET_MARGIN_POSITIONS")]
    GetMarginPositions { payload: MarginPositionsPayload },
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserBalancesPayload {
    pub balances: Vec<Balances>,
}

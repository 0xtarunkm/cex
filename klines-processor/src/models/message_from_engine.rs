use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum MessageFromEngine {
    #[serde(rename = "TRADE_ADDED")]
    CreateOrder { data: AddTradePayload },
}

#[derive(Debug, Deserialize)]
pub struct AddTradePayload {
    pub time: DateTime<Utc>,
    pub price: Decimal
}
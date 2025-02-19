use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename = "TRADE_ADDED")]
pub struct AddTradePayload {
    pub data: TradeData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradeData {
    pub ticker: String,
    pub time: DateTime<Utc>,
    pub price: Decimal,
}

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
    pub ticker: Ticker,
    pub time: DateTime<Utc>,
    pub price: Decimal,
}

#[derive(Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Ticker {
    SOL_USDC,
    ETH_USDC,
    BTC_USDC,
}

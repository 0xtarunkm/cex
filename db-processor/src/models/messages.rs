use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageFromEngine {
    #[serde(rename = "TRADE_ADDED")]
    TradeAdded { data: TradeAddedType },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeAddedType {
    pub market: String,
    pub price: f64,
    pub timestamp: String,
}

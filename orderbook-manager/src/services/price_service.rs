use rust_decimal::Decimal;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct PriceInfo {
    pub last_trade_price: Option<Decimal>,
    pub mark_price: Decimal,
    pub index_price: Option<Decimal>,
    pub timestamp: i64,
}

#[allow(dead_code)]
pub struct PriceService {
    prices: Arc<RwLock<HashMap<String, PriceInfo>>>,
}

impl PriceService {
    pub fn new() -> Self {
        PriceService {
            prices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn update_price(&self, market: &str, price_info: PriceInfo) {
        let mut prices = self.prices.write().await;
        // info!(?market, ?price_info, "Updating price");
        prices.insert(market.to_string(), price_info);
    }

    pub async fn get_price(&self, market: &str) -> Option<Decimal> {
        let prices = self.prices.read().await;
        prices.get(market).map(|info| info.mark_price)
    }
}

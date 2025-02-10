use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::Utc;
use rust_decimal::Decimal;
use tokio::sync::Mutex;

use crate::trade::{MarginOrderbook, SpotOrderbook};

#[derive(Debug, Clone)]
pub struct PriceInfo {
    pub last_trade_price: Option<Decimal>,
    pub mark_price: Decimal,
    pub index_price: Option<Decimal>,
    pub timestamp: i64,
}

pub struct PriceService {
    prices: Arc<Mutex<HashMap<String, PriceInfo>>>,
    spot_orderbooks: Arc<Mutex<HashMap<String, Arc<Mutex<SpotOrderbook>>>>>,
    margin_orderbooks: Arc<Mutex<HashMap<String, Arc<Mutex<MarginOrderbook>>>>>,
}

impl PriceService {
    pub fn new(
        spot_orderbooks: Arc<Mutex<HashMap<String, Arc<Mutex<SpotOrderbook>>>>>,
        margin_orderbooks: Arc<Mutex<HashMap<String, Arc<Mutex<MarginOrderbook>>>>>,
    ) -> Self {
        PriceService {
            prices: Arc::new(Mutex::new(HashMap::new())),
            spot_orderbooks,
            margin_orderbooks,
        }
    }

    pub async fn update_trade_price(&self, market: &str, price: Decimal) {
        let mut prices = self.prices.lock().await;
        let now = Utc::now().timestamp();

        prices.insert(
            market.to_string(),
            PriceInfo {
                last_trade_price: Some(price),
                mark_price: price,
                index_price: None,
                timestamp: now,
            },
        );
    }

    async fn calculate_mid_price(&self, market: &str) -> Option<Decimal> {
        let spot_orderbooks = self.spot_orderbooks.lock().await;
        if let Some(orderbook) = spot_orderbooks.get(market) {
            let ob = orderbook.lock().await;
            if !ob.bids.is_empty() && !ob.asks.is_empty() {
                let best_bid = ob.bids[0].price;
                let best_ask = ob.asks[0].price;
                return Some((best_bid + best_ask) / Decimal::from(2));
            }
        }
        None
    }

    pub async fn start_price_updates(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            self.update_all_prices().await;
        }
    }

    async fn update_all_prices(&self) {
        let markets = vec!["SOL_USDC", "BTC_USDC", "ETH_USDC"];

        for market in markets {
            if let Some(mid_price) = self.calculate_mid_price(market).await {
                self.update_trade_price(&market, mid_price).await;
            }
        }
    }

    pub async fn get_price(&self, market: &str) -> Option<Decimal> {
        let prices = self.prices.lock().await;
        prices.get(market).map(|info| info.mark_price)
    }

    pub async fn get_price_info(&self, market: &str) -> Option<PriceInfo> {
        let prices = self.prices.lock().await;
        prices.get(market).cloned()
    }
}

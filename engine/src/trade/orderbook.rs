use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::BASE_CURRENCY;

// Order Side Enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

// Order Struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub price: f64,
    pub quantity: f64,
    pub order_id: String,
    pub filled: f64,
    pub side: OrderSide,
    pub user_id: String,
}

// Fill Struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub price: String,
    pub qty: f64,
    pub trade_id: u64,
    pub other_user_id: String,
    pub marker_order_id: String,
}

// Orderbook Struct (assumed to be defined elsewhere)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub base_asset: String,
    pub quote_asset: String,
    pub last_trade_id: u64,
    pub current_price: f64,
}

impl Orderbook {
    pub fn new(
        base_asset: String,
        bids: Vec<Order>,
        asks: Vec<Order>,
        last_trade_id: u64,
        current_price: f64,
    ) -> Self {
        Orderbook {
            bids,
            asks,
            base_asset,
            quote_asset: BASE_CURRENCY.to_string(),
            last_trade_id,
            current_price,
        }
    }

    pub fn ticker(&self) -> String {
        format!("{}_{}", self.base_asset, self.quote_asset)
    }

    pub fn get_snapshot(&self) -> OrderbookSnapshot {
        OrderbookSnapshot {
            base_asset: self.base_asset.clone(),
            bids: self.bids.clone(),
            asks: self.asks.clone(),
            last_trade_id: self.last_trade_id,
            current_price: self.current_price,
        }
    }

    pub fn add_order(&mut self, order: Order) -> OrderExecutionResult {
        let (executed_qty, fills) = match order.side {
            OrderSide::Buy => self.match_bid(order.clone()),
            OrderSide::Sell => self.match_ask(order.clone()),
        };

        let mut order = order;
        order.filled = executed_qty;

        if executed_qty < order.quantity {
            match order.side {
                OrderSide::Buy => self.bids.push(order),
                OrderSide::Sell => self.asks.push(order),
            }
        }

        OrderExecutionResult {
            executed_qty,
            fills,
        }
    }

    pub fn match_bid(&mut self, order: Order) -> (f64, Vec<Fill>) {
        let mut fills = Vec::new();
        let mut executed_qty = 0.0;

        for ask in &mut self.asks {
            if ask.price <= order.price && executed_qty < order.quantity {
                let filled_qty = (order.quantity - executed_qty).min(ask.quantity);
                executed_qty += filled_qty;
                ask.filled += filled_qty;

                fills.push(Fill {
                    price: ask.price.to_string(),
                    qty: filled_qty,
                    trade_id: self.last_trade_id,
                    other_user_id: ask.user_id.clone(),
                    marker_order_id: ask.order_id.clone(),
                });

                self.last_trade_id += 1;
            }
        }

        self.asks.retain(|ask| ask.filled < ask.quantity);

        (executed_qty, fills)
    }

    pub fn match_ask(&mut self, order: Order) -> (f64, Vec<Fill>) {
        let mut fills = Vec::new();
        let mut executed_qty = 0.0;

        for bid in &mut self.bids {
            if bid.price >= order.price && executed_qty < order.quantity {
                let filled_qty = (order.quantity - executed_qty).min(bid.quantity);
                executed_qty += filled_qty;
                bid.filled += filled_qty;

                fills.push(Fill {
                    price: bid.price.to_string(),
                    qty: filled_qty,
                    trade_id: self.last_trade_id,
                    other_user_id: bid.user_id.clone(),
                    marker_order_id: bid.order_id.clone(),
                });

                self.last_trade_id += 1;
            }
        }

        self.bids.retain(|bid| bid.filled < bid.quantity);

        (executed_qty, fills)
    }

    pub fn get_depth(&self) -> Depth {
        let mut bids_map: HashMap<String, f64> = HashMap::new();
        let mut asks_map: HashMap<String, f64> = HashMap::new();

        for bid in &self.bids {
            *bids_map.entry(bid.price.to_string()).or_insert(0.0) += bid.quantity;
        }

        for ask in &self.asks {
            *asks_map.entry(ask.price.to_string()).or_insert(0.0) += ask.quantity;
        }

        let bids: Vec<(String, String)> = bids_map
            .into_iter()
            .map(|(price, qty)| (price, qty.to_string()))
            .collect();

        let asks: Vec<(String, String)> = asks_map
            .into_iter()
            .map(|(price, qty)| (price, qty.to_string()))
            .collect();

        Depth { bids, asks }
    }

    pub fn get_open_orders(&self, user_id: &str) -> Vec<Order> {
        let asks = self
            .asks
            .iter()
            .filter(|x| x.user_id == user_id)
            .cloned()
            .collect::<Vec<_>>();
        let bids = self
            .bids
            .iter()
            .filter(|x| x.user_id == user_id)
            .cloned()
            .collect::<Vec<_>>();
        [asks, bids].concat()
    }

    pub fn cancel_bid(&mut self, order_id: &str) -> Option<f64> {
        if let Some(index) = self.bids.iter().position(|x| x.order_id == order_id) {
            let price = self.bids[index].price;
            self.bids.remove(index);
            Some(price)
        } else {
            None
        }
    }

    pub fn cancel_ask(&mut self, order_id: &str) -> Option<f64> {
        if let Some(index) = self.asks.iter().position(|x| x.order_id == order_id) {
            let price = self.asks[index].price;
            self.asks.remove(index);
            Some(price)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSnapshot {
    pub base_asset: String,
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub last_trade_id: u64,
    pub current_price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderExecutionResult {
    pub executed_qty: f64,
    pub fills: Vec<Fill>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Depth {
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

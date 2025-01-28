use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::models::{DepthPayload, Fill, MatchResult, Order, OrderSide};

pub struct Orderbook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub base_asset: String,
    pub quote_asset: String,
    pub last_trade_id: u64,
}

impl Orderbook {
    pub fn new(&self, base_asset: String, quote_asset: String) -> Self {
        Orderbook {
            bids: Vec::new(),
            asks: Vec::new(),
            base_asset,
            quote_asset,
            last_trade_id: 1,
        }
    }

    pub fn ticker(&self) -> String {
        format!("{}_{}", self.base_asset, self.quote_asset)
    }

    pub fn add_order(&mut self, mut order: Order) -> MatchResult {
        match order.side {
            OrderSide::Buy => {
                let match_result = self.match_bid(&order);
                order.filled = match_result.executed_qty;

                if order.filled < order.quantity {
                    self.bids.push(order);
                }

                match_result
            }
            OrderSide::Sell => {
                let match_result = self.match_ask(&order);
                order.filled = match_result.executed_qty;

                if order.filled < order.quantity {
                    self.asks.push(order);
                }

                match_result
            }
        }
    }

    fn match_bid(&mut self, order: &Order) -> MatchResult {
        let mut fills = Vec::new();
        let mut executed_qty = Decimal::from(0);

        let mut i = 0;
        while i < self.asks.len() {
            if self.asks[i].price <= order.price && executed_qty < order.quantity {
                let remaining = order.quantity - executed_qty;
                let fill_qty =
                    std::cmp::min(remaining, self.asks[i].quantity - self.asks[i].filled);

                if fill_qty > Decimal::from(0) {
                    executed_qty += fill_qty;
                    self.asks[i].filled += fill_qty;

                    fills.push(Fill {
                        price: self.asks[i].price.to_string(),
                        quantity: fill_qty,
                        trade_id: self.last_trade_id,
                        other_user_id: self.asks[i].user_id.clone(),
                        marker_order_id: self.asks[i].order_id.clone(),
                    });

                    self.last_trade_id += 1;
                }
            }
            i += 1;
        }

        self.asks.retain(|ask| ask.filled < ask.quantity);

        MatchResult {
            executed_qty,
            fills,
        }
    }

    fn match_ask(&mut self, order: &Order) -> MatchResult {
        let mut fills = Vec::new();
        let mut executed_qty = Decimal::from(0);

        let mut i = 0;
        while i < self.bids.len() {
            if self.bids[i].price >= order.price && executed_qty < order.quantity {
                let remaining = order.quantity - executed_qty;
                let fill_qty =
                    std::cmp::min(remaining, self.bids[i].quantity - self.bids[i].filled);

                if fill_qty > Decimal::from(0) {
                    executed_qty += fill_qty;
                    self.bids[i].filled += fill_qty;

                    fills.push(Fill {
                        price: self.bids[i].price.to_string(),
                        quantity: fill_qty,
                        trade_id: self.last_trade_id,
                        other_user_id: self.bids[i].user_id.clone(),
                        marker_order_id: self.bids[i].order_id.clone(),
                    });

                    self.last_trade_id += 1;
                }
            }
            i += 1;
        }

        self.bids.retain(|bid| bid.filled < bid.quantity);

        MatchResult {
            executed_qty,
            fills,
        }
    }

    pub fn get_depth(&self) -> DepthPayload {
        let mut bids_obj: HashMap<Decimal, Decimal> = HashMap::new();
        let mut asks_obj: HashMap<Decimal, Decimal> = HashMap::new();

        for order in &self.bids {
            *bids_obj.entry(order.price).or_insert(Decimal::new(0, 0)) += order.quantity;
        }

        for order in &self.asks {
            *asks_obj.entry(order.price).or_insert(Decimal::new(0, 0)) += order.quantity;
        }

        let bids: Vec<(String, String)> = bids_obj
            .into_iter()
            .map(|(price, quantity)| (price.to_string(), quantity.to_string()))
            .collect();

        let asks: Vec<(String, String)> = asks_obj
            .into_iter()
            .map(|(price, quantity)| (price.to_string(), quantity.to_string()))
            .collect();

        DepthPayload { bids, asks }
    }

    pub fn cancel_bid(&mut self, order: &Order) -> Option<Decimal> {
        if let Some(index) = self.bids.iter().position(|x| x.order_id == order.order_id) {
            let price = self.bids[index].price;
            self.bids.remove(index);
            Some(price)
        } else {
            None
        }
    }

    pub fn cancel_ask(&mut self, order: &Order) -> Option<Decimal> {
        if let Some(index) = self.asks.iter().position(|x| x.order_id == order.order_id) {
            let price = self.asks[index].price;
            self.asks.remove(index);
            Some(price)
        } else {
            None
        }
    }

    pub fn get_open_orders(&mut self, user_id: String) -> Vec<Order> {
        let mut orders: Vec<Order> = self
            .asks
            .iter()
            .filter(|x| x.user_id == user_id)
            .cloned()
            .collect();

        let bids: Vec<Order> = self
            .bids
            .iter()
            .filter(|x| x.user_id == user_id)
            .cloned()
            .collect();

        orders.extend(bids);
        orders
    }
}

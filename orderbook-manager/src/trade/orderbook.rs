// use crate::models::{DepthPayload, Fill, MatchResult, Order, OrderSide};
// use rust_decimal::Decimal;
// use std::collections::HashMap;

use std::sync::{Arc, Mutex};

use crate::models::Order;

pub struct Orderbook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub base_asset: String,
    pub quote_asset: String,
}

impl Orderbook {
    pub fn _new(&self, base_asset: String, quote_asset: String) -> Self {
        Orderbook {
            bids: Vec::new(),
            asks: Vec::new(),
            base_asset,
            quote_asset,
        }
    }

    pub fn ticker(&self) -> String {
        format!("{}_{}", self.base_asset, self.quote_asset)
    }

    pub fn add_order(&mut self, mut order: Order) -> MatchResult {
        match order.side {
            OrderSide::Buy => {
                self.asks
                    .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                let match_result = self.match_bid(&order);
                order.filled = match_result.executed_qty;

                if order.filled < order.quantity {
                    self.bids.push(order);
                }

                match_result
            }
            OrderSide::Sell => {
                self.bids
                    .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
                let match_result = self.match_ask(&order);
                order.filled = match_result.executed_qty;

                if order.filled < order.quantity {
                    self.asks.push(order);
                }

                match_result
            }
        }
    }

//     fn match_bid(&mut self, order: &Order) -> MatchResult {
//         let mut fills = Vec::new();
//         let mut executed_qty = Decimal::from(0);

//         let mut i = 0;
//         while i < self.asks.len() {
//             if self.asks[i].price <= order.price && executed_qty < order.quantity {
//                 let remaining = order.quantity - executed_qty;
//                 let fill_qty =
//                     std::cmp::min(remaining, self.asks[i].quantity - self.asks[i].filled);

//                 if fill_qty > Decimal::from(0) {
//                     executed_qty += fill_qty;
//                     self.asks[i].filled += fill_qty;

//                     fills.push(Fill {
//                         price: self.asks[i].price,
//                         quantity: fill_qty,
//                         trade_id: self.last_trade_id,
//                         other_user_id: self.asks[i].user_id.clone(),
//                         marker_order_id: self.asks[i].order_id.clone(),
//                     });

//                     self.last_trade_id += 1;
//                 }
//             }
//             i += 1;
//         }

//         self.asks.retain(|ask| ask.filled < ask.quantity);

//         MatchResult {
//             executed_qty,
//             fills,
//         }
//     }

//     fn match_ask(&mut self, order: &Order) -> MatchResult {
//         let mut fills = Vec::new();
//         let mut executed_qty = Decimal::from(0);

//         let mut i = 0;
//         while i < self.bids.len() {
//             if self.bids[i].price >= order.price && executed_qty < order.quantity {
//                 let remaining = order.quantity - executed_qty;
//                 let fill_qty =
//                     std::cmp::min(remaining, self.bids[i].quantity - self.bids[i].filled);

//                 if fill_qty > Decimal::from(0) {
//                     executed_qty += fill_qty;
//                     self.bids[i].filled += fill_qty;

//                     fills.push(Fill {
//                         price: self.bids[i].price,
//                         quantity: fill_qty,
//                         trade_id: self.last_trade_id,
//                         other_user_id: self.bids[i].user_id.clone(),
//                         marker_order_id: self.bids[i].order_id.clone(),
//                     });

//                     self.last_trade_id += 1;
//                 }
//             }
//             i += 1;
//         }

//         self.bids.retain(|bid| bid.filled < bid.quantity);

//         MatchResult {
//             executed_qty,
//             fills,
//         }
//     }

//     pub fn get_depth(&self) -> DepthPayload {
//         let mut bids_obj: HashMap<Decimal, Decimal> = HashMap::new();
//         let mut asks_obj: HashMap<Decimal, Decimal> = HashMap::new();

//         for order in &self.bids {
//             let remaining_qty = order.quantity - order.filled;
//             if remaining_qty > Decimal::from(0) {
//                 *bids_obj.entry(order.price).or_insert(Decimal::new(0, 0)) += remaining_qty;
//             }
//         }

//         for order in &self.asks {
//             let remaining_qty = order.quantity - order.filled;
//             if remaining_qty > Decimal::from(0) {
//                 *asks_obj.entry(order.price).or_insert(Decimal::new(0, 0)) += remaining_qty;
//             }
//         }

//         let bids: Vec<(Decimal, Decimal)> = bids_obj
//             .into_iter()
//             .map(|(price, quantity)| (price, quantity))
//             .collect();

//         let asks: Vec<(Decimal, Decimal)> = asks_obj
//             .into_iter()
//             .map(|(price, quantity)| (price, quantity))
//             .collect();

//         DepthPayload { bids, asks }
//     }

//     pub fn get_open_orders(&mut self, user_id: String) -> Vec<Order> {
//         let mut orders: Vec<Order> = self
//             .asks
//             .iter()
//             .filter(|x| x.user_id == user_id)
//             .cloned()
//             .collect();

//         let bids: Vec<Order> = self
//             .bids
//             .iter()
//             .filter(|x| x.user_id == user_id)
//             .cloned()
//             .collect();

//         orders.extend(bids);
//         orders
//     }
}

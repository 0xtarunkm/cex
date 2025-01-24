use crate::models::{Fill, Order, Orderbook, Side};
use rust_decimal::Decimal;
use std::collections::BTreeMap;

impl Orderbook {
    pub fn new(
        base_asset: String,
        bids: Vec<Order>,
        asks: Vec<Order>,
        last_trade_id: u64,
        current_price: Decimal,
    ) -> Self {
        let mut bid_map = BTreeMap::new();
        let mut ask_map = BTreeMap::new();

        for bid in bids {
            bid_map.entry(bid.price).or_insert_with(Vec::new).push(bid);
        }

        for ask in asks {
            ask_map.entry(ask.price).or_insert_with(Vec::new).push(ask);
        }

        Orderbook {
            bids: bid_map,
            asks: ask_map,
            base_asset,
            quote_asset: "INR".to_string(),
            last_trade_id,
            current_price,
        }
    }

    pub fn ticker(&self) -> String {
        format!("{}_{}", self.base_asset, self.quote_asset)
    }

    pub fn add_order(&mut self, order: Order) -> Result<(Vec<Fill>, Decimal), String> {
        let mut fills = Vec::new();
        let mut executed_qty = Decimal::ZERO;

        match order.side {
            Side::Buy => {
                for (price, asks) in self.asks.iter_mut() {
                    if *price > order.price || executed_qty >= order.quantity {
                        break;
                    }

                    for ask in asks.iter_mut() {
                        if executed_qty >= order.quantity {
                            break;
                        }

                        let filled_qty = (order.quantity - executed_qty).min(ask.quantity);
                        executed_qty += filled_qty;
                        ask.filled += filled_qty;

                        fills.push(Fill {
                            price: *price,
                            qty: filled_qty,
                            trade_id: self.last_trade_id,
                            other_user_id: ask.user_id.clone(),
                            marker_order_id: ask.order_id.clone(),
                        });

                        self.last_trade_id += 1;
                    }
                }

                if executed_qty < order.quantity {
                    self.bids
                        .entry(order.price)
                        .or_insert_with(Vec::new)
                        .push(order);
                }
            }
            Side::Sell => {
                for (price, bids) in self.bids.iter_mut().rev() {
                    if *price < order.price || executed_qty >= order.quantity {
                        break;
                    }

                    for bid in bids.iter_mut() {
                        if executed_qty >= order.quantity {
                            break;
                        }

                        let filled_qty = (order.quantity - executed_qty).min(bid.quantity);
                        executed_qty += filled_qty;
                        bid.filled += filled_qty;

                        fills.push(Fill {
                            price: *price,
                            qty: filled_qty,
                            trade_id: self.last_trade_id,
                            other_user_id: bid.user_id.clone(),
                            marker_order_id: bid.order_id.clone(),
                        });

                        self.last_trade_id += 1;
                    }
                }

                if executed_qty < order.quantity {
                    self.asks
                        .entry(order.price)
                        .or_insert_with(Vec::new)
                        .push(order);
                }
            }
        }

        Ok((fills, executed_qty))
    }

    pub fn cancel_bid(&mut self, order_id: &str) -> Option<Decimal> {
        let mut canceled_price = None;

        self.bids.retain(|_, bids| {
            if let Some(index) = bids.iter().position(|bid| bid.order_id == order_id) {
                canceled_price = Some(bids[index].price);
                bids.remove(index);
            }
            // Only retain this entry if `bids` is not empty
            !bids.is_empty()
        });

        canceled_price
    }

    pub fn cancel_ask(&mut self, order_id: &str) -> Option<Decimal> {
        let mut canceled_price = None;

        self.asks.retain(|_, asks| {
            if let Some(index) = asks.iter().position(|ask| ask.order_id == order_id) {
                canceled_price = Some(asks[index].price);
                asks.remove(index);
            }

            !asks.is_empty()
        });

        canceled_price
    }
}

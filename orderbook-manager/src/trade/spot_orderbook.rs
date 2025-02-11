use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tokio::sync::Mutex;

use crate::{models::{
    CreateOrderPayload, Depth, GetQuoteResponse, OrderDetails, OrderSide, SpotOrder, User,
}, services::price_service::PriceInfo};

#[allow(dead_code)]
pub struct SpotOrderbook {
    pub bids: Vec<SpotOrder>,
    pub asks: Vec<SpotOrder>,
    pub base_asset: String,
    pub quote_asset: String,
}

impl SpotOrderbook {
    pub fn new(base_asset: String, quote_asset: String) -> Self {
        SpotOrderbook {
            bids: Vec::new(),
            asks: Vec::new(),
            base_asset,
            quote_asset,
        }
    }

    pub async fn fill_orders(
        &mut self,
        order: &CreateOrderPayload,
        users: &mut Arc<Mutex<Vec<User>>>,
        base_asset: &str,
        quote_asset: &str,
    ) -> Decimal {
        let mut remaining_qty = order.quantity;

        match order.side {
            OrderSide::Buy => {
                while !self.asks.is_empty() && remaining_qty > Decimal::ZERO {
                    let ask = &self.asks[0];

                    if ask.price > order.price {
                        break;
                    }

                    let fill_qty = remaining_qty.min(ask.quantity);

                    self.flip_balance(
                        &order.user_id,
                        &ask.user_id,
                        ask.price,
                        fill_qty,
                        users,
                        base_asset,
                        quote_asset,
                    )
                    .await;

                    remaining_qty -= fill_qty;
                    self.asks[0].quantity -= fill_qty;

                    if self.asks[0].quantity == Decimal::ZERO {
                        self.asks.remove(0);
                    }
                }
            }
            OrderSide::Sell => {
                while !self.bids.is_empty() && remaining_qty > Decimal::ZERO {
                    let bid = &self.bids[0];

                    if bid.price < order.price {
                        break;
                    }

                    let fill_qty = remaining_qty.min(bid.quantity);

                    self.flip_balance(
                        &bid.user_id,
                        &order.user_id,
                        bid.price,
                        fill_qty,
                        users,
                        base_asset,
                        quote_asset,
                    )
                    .await;

                    remaining_qty -= fill_qty;
                    self.bids[0].quantity -= fill_qty;

                    if self.bids[0].quantity == Decimal::ZERO {
                        self.bids.remove(0);
                    }
                }
            }
        }

        remaining_qty
    }

    pub fn get_depth(&self) -> Depth {
        let mut depth: HashMap<Decimal, OrderDetails> = HashMap::new();

        // Process bids
        for bid in &self.bids {
            depth
                .entry(bid.price)
                .and_modify(|details| {
                    details.quantity += bid.quantity;
                })
                .or_insert(OrderDetails {
                    type_: OrderSide::Buy,
                    quantity: bid.quantity,
                });
        }

        // Process asks
        for ask in &self.asks {
            depth
                .entry(ask.price)
                .and_modify(|details| {
                    details.quantity += ask.quantity;
                })
                .or_insert(OrderDetails {
                    type_: OrderSide::Sell,
                    quantity: ask.quantity,
                });
        }

        Depth { orders: depth }
    }

    pub fn get_quote_detail(&self, quantity: Decimal, side: OrderSide) -> GetQuoteResponse {
        let mut remaining_qty = quantity;
        let mut total_cost = Decimal::from(0);
        let mut weighted_avg_price = Decimal::from(0);

        match side {
            OrderSide::Buy => {
                for ask in self.asks.iter() {
                    if remaining_qty == Decimal::from(0) {
                        break;
                    }

                    if remaining_qty > ask.quantity {
                        total_cost += ask.price * ask.quantity;
                        remaining_qty -= ask.quantity;
                    } else {
                        total_cost += ask.price * remaining_qty;
                        remaining_qty = Decimal::from(0);
                        break;
                    }
                }
            }
            OrderSide::Sell => {
                for bid in self.bids.iter() {
                    if remaining_qty == Decimal::from(0) {
                        break;
                    }
                    if remaining_qty >= bid.quantity {
                        total_cost += bid.price * bid.quantity;
                        remaining_qty -= bid.quantity;
                    } else {
                        total_cost += bid.price * remaining_qty;
                        remaining_qty = Decimal::from(0);
                        break;
                    }
                }
            }
        }

        if remaining_qty < quantity {
            weighted_avg_price = total_cost / (quantity - remaining_qty);
        }

        GetQuoteResponse {
            avg_price: weighted_avg_price,
            quantity,
            total_cost,
        }
    }

    async fn flip_balance(
        &self,
        buyer_id: &str,
        seller_id: &str,
        price: Decimal,
        quantity: Decimal,
        users: &mut Arc<Mutex<Vec<User>>>,
        base_asset: &str,
        quote_asset: &str,
    ) {
        let mut users_guard = users.lock().await;
        let trade_value = price * quantity;

        if let Some(seller) = users_guard.iter_mut().find(|u| u.id == seller_id) {
            if let Some(base_balance) = seller.balances.iter_mut().find(|b| b.ticker == base_asset)
            {
                base_balance.locked_balance =
                    base_balance.locked_balance.checked_sub(quantity).unwrap();
                base_balance.balance = base_balance.balance.checked_sub(quantity).unwrap();
            }

            if let Some(quote_balance) =
                seller.balances.iter_mut().find(|b| b.ticker == quote_asset)
            {
                quote_balance.balance = quote_balance.balance.checked_add(trade_value).unwrap();
            }
        }

        if let Some(buyer) = users_guard.iter_mut().find(|u| u.id == buyer_id) {
            if let Some(base_balance) = buyer.balances.iter_mut().find(|b| b.ticker == base_asset) {
                base_balance.balance = base_balance.balance.checked_add(quantity).unwrap();
            }

            if let Some(quote_balance) = buyer.balances.iter_mut().find(|b| b.ticker == quote_asset)
            {
                quote_balance.locked_balance = quote_balance
                    .locked_balance
                    .checked_sub(trade_value)
                    .unwrap();
                quote_balance.balance = quote_balance.balance.checked_sub(trade_value).unwrap();
            }
        }
    }

    pub async fn get_price_info(&self) -> Option<PriceInfo> {
        let now = Utc::now().timestamp();
        if self.bids.is_empty() && self.asks.is_empty() {
            return Some(PriceInfo {
                last_trade_price: None,
                mark_price: dec!(100),
                index_price: None,
                timestamp: now,
            });
        }
        let best_bid = self.bids.first().map(|o| o.price);
        let best_ask = self.asks.first().map(|o| o.price);
        let mark_price = match (best_bid, best_ask) {
            (Some(b), Some(a)) => (b + a) / dec!(2),
            (Some(b), None) => b,
            (None, Some(a)) => a,
            (None, None) => dec!(100),
        };
        Some(PriceInfo {
            last_trade_price: None,
            mark_price,
            index_price: None,
            timestamp: now,
        })
    }
}

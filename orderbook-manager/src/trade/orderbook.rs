use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tokio::sync::Mutex;

use crate::{
    models::{
        CreateOrderPayload, Depth, GetQuoteResponse, MarginPosition, Order, OrderDetails,
        OrderSide, PositionType, User,
    },
    services::price_service::PriceInfo,
};

#[allow(dead_code)]
pub struct Orderbook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub base_asset: String,
    pub quote_asset: String,
}

impl Orderbook {
    pub fn new(base_asset: String, quote_asset: String) -> Self {
        Orderbook {
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
                for i in 0..self.asks.len() {
                    if self.asks[i].price > order.price {
                        break;
                    }

                    let match_qty = if self.asks[i].quantity > remaining_qty {
                        remaining_qty
                    } else {
                        self.asks[i].quantity
                    };

                    match (order.is_margin, self.asks[i].is_margin) {
                        // Margin buy vs Margin Sell
                        (true, true) => {
                            let buyer_position = MarginPosition {
                                asset: format!("{}_{}", self.base_asset, self.quote_asset),
                                user_id: order.user_id.clone(),
                                position_type: PositionType::Long,
                                entry_price: self.asks[i].price,
                                size: match_qty,
                                leverage: order.leverage.unwrap(),
                                collateral: self.calculate_required_margin(
                                    self.asks[i].price,
                                    match_qty,
                                    order.leverage.unwrap(),
                                ),
                                unrealized_pnl: dec!(0),
                            };

                            let seller_position = MarginPosition {
                                asset: format!("{}_{}", self.base_asset, self.quote_asset),
                                user_id: order.user_id.clone(),
                                position_type: PositionType::Short,
                                entry_price: self.asks[i].price,
                                size: match_qty,
                                leverage: self.asks[i].leverage.unwrap(),
                                collateral: self.calculate_required_margin(
                                    self.asks[i].price,
                                    match_qty,
                                    self.asks[i].leverage.unwrap(),
                                ),
                                unrealized_pnl: dec!(0),
                            };

                            self.net_position(users, &order.user_id, buyer_position)
                                .await
                                .unwrap();
                            self.net_position(users, &order.user_id, seller_position)
                                .await
                                .unwrap();
                        }
                        // Margin Buy vs Spot Sell
                        (true, false) => {
                            let buyer_position = MarginPosition {
                                asset: format!("{}_{}", self.base_asset, self.quote_asset),
                                user_id: order.user_id.clone(),
                                position_type: PositionType::Long,
                                entry_price: self.asks[i].price,
                                size: match_qty,
                                leverage: order.leverage.unwrap(),
                                collateral: self.calculate_required_margin(
                                    self.asks[i].price,
                                    match_qty,
                                    order.leverage.unwrap(),
                                ),
                                unrealized_pnl: dec!(0),
                            };
                            self.net_position(users, &order.user_id, buyer_position)
                                .await
                                .unwrap();

                            self.flip_balance(
                                &order.user_id,
                                &self.asks[i].user_id,
                                self.asks[i].price,
                                match_qty,
                                users,
                                base_asset,
                                quote_asset,
                            )
                            .await;
                        }
                        // Spot buy vs Margin sell
                        (false, true) => {
                            self.flip_balance(
                                &order.user_id,
                                &self.asks[i].user_id,
                                self.asks[i].price,
                                match_qty,
                                users,
                                base_asset,
                                quote_asset,
                            )
                            .await;

                            let seller_position = MarginPosition {
                                asset: format!("{}_{}", self.base_asset, self.quote_asset),
                                user_id: self.asks[i].user_id.clone(),
                                position_type: PositionType::Short,
                                entry_price: self.asks[i].price,
                                size: match_qty,
                                leverage: self.asks[i].leverage.unwrap(),
                                collateral: self.calculate_required_margin(
                                    self.asks[i].price,
                                    match_qty,
                                    self.asks[i].leverage.unwrap(),
                                ),
                                unrealized_pnl: dec!(0),
                            };
                            self.net_position(users, &self.asks[i].user_id, seller_position)
                                .await
                                .unwrap();
                        }
                        // Spot Buy vs Spot Sell
                        (false, false) => {
                            self.flip_balance(
                                &order.user_id,
                                &self.asks[i].user_id,
                                self.asks[i].price,
                                match_qty,
                                users,
                                base_asset,
                                quote_asset,
                            )
                            .await;
                        }
                    }
                    remaining_qty -= match_qty;
                    if self.asks[i].quantity == match_qty {
                        self.asks.remove(i);
                    } else {
                        self.asks[i].quantity -= match_qty;
                    }

                    if remaining_qty == dec!(0) {
                        break;
                    }
                }
            }
            OrderSide::Sell => {
                for i in 0..self.bids.len() {
                    if self.bids[i].price < order.price {
                        break;
                    }

                    let match_qty = if self.bids[i].quantity > remaining_qty {
                        remaining_qty
                    } else {
                        self.bids[i].quantity
                    };

                    match (order.is_margin, self.bids[i].is_margin) {
                        // margin sell vs margin buy
                        (true, true) => {
                            let seller_position = MarginPosition {
                                asset: format!("{}_{}", self.base_asset, self.quote_asset),
                                user_id: order.user_id.clone(),
                                position_type: PositionType::Short,
                                entry_price: self.bids[i].price,
                                size: match_qty,
                                leverage: order.leverage.unwrap(),
                                collateral: self.calculate_required_margin(
                                    self.bids[i].price,
                                    match_qty,
                                    order.leverage.unwrap(),
                                ),
                                unrealized_pnl: dec!(0),
                            };

                            let buyer_position = MarginPosition {
                                asset: format!("{}_{}", self.base_asset, self.quote_asset),
                                user_id: self.bids[i].user_id.clone(),
                                position_type: PositionType::Long,
                                entry_price: self.bids[i].price,
                                size: match_qty,
                                leverage: self.bids[i].leverage.unwrap(),
                                collateral: self.calculate_required_margin(
                                    self.bids[i].price,
                                    match_qty,
                                    self.bids[i].leverage.unwrap(),
                                ),
                                unrealized_pnl: dec!(0),
                            };

                            self.net_position(users, &order.user_id, seller_position)
                                .await
                                .unwrap();
                            self.net_position(users, &self.bids[i].user_id, buyer_position)
                                .await
                                .unwrap();
                        }
                        // margin sell vs spot buy
                        (true, false) => {
                            let seller_position = MarginPosition {
                                asset: format!("{}_{}", self.base_asset, self.quote_asset),
                                user_id: order.user_id.to_string(),
                                position_type: PositionType::Short,
                                entry_price: self.bids[i].price,
                                size: match_qty,
                                leverage: order.leverage.unwrap(),
                                collateral: self.calculate_required_margin(
                                    self.bids[i].price,
                                    match_qty,
                                    order.leverage.unwrap(),
                                ),
                                unrealized_pnl: dec!(0),
                            };
                            self.net_position(users, &order.user_id, seller_position)
                                .await
                                .unwrap();

                            self.flip_balance(
                                &self.bids[i].user_id,
                                &order.user_id,
                                self.bids[i].price,
                                match_qty,
                                users,
                                base_asset,
                                quote_asset,
                            )
                            .await;
                        }
                        // spot sell vs margin buy
                        (false, true) => {
                            self.flip_balance(
                                &self.bids[i].user_id,
                                &order.user_id,
                                self.bids[i].price,
                                match_qty,
                                users,
                                base_asset,
                                quote_asset,
                            )
                            .await;

                            let buyer_position = MarginPosition {
                                asset: format!("{}_{}", self.base_asset, self.quote_asset),
                                user_id: self.bids[i].user_id.clone(),
                                position_type: PositionType::Long,
                                entry_price: self.bids[i].price,
                                size: match_qty,
                                leverage: self.bids[i].leverage.unwrap(),
                                collateral: self.calculate_required_margin(
                                    self.bids[i].price,
                                    match_qty,
                                    self.bids[i].leverage.unwrap(),
                                ),
                                unrealized_pnl: dec!(0),
                            };

                            self.net_position(users, &self.bids[i].user_id, buyer_position)
                                .await
                                .unwrap();
                        }
                        (false, false) => {
                            self.flip_balance(
                                &self.bids[i].user_id,
                                &order.user_id,
                                self.bids[i].price,
                                match_qty,
                                users,
                                base_asset,
                                quote_asset,
                            )
                            .await;
                        }
                    }
                    remaining_qty -= match_qty;
                    if self.bids[i].quantity == match_qty {
                        self.bids.remove(i);
                    } else {
                        self.bids[i].quantity -= match_qty;
                    }

                    if remaining_qty == dec!(0) {
                        break;
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

    fn calculate_required_margin(
        &self,
        price: Decimal,
        quantity: Decimal,
        leverage: Decimal,
    ) -> Decimal {
        (price * quantity) / Decimal::from(leverage)
    }

    async fn net_position(
        &self,
        users: &mut Arc<Mutex<Vec<User>>>,
        user_id: &str,
        new_position: MarginPosition,
    ) -> Result<(), &'static str> {
        let mut user_guard = users.lock().await;
        match user_guard.iter_mut().find(|u| u.id == user_id) {
            Some(user) => {
                match user.margin_positions.iter_mut().find(|p| {
                    p.asset == new_position.asset && p.position_type == new_position.position_type
                }) {
                    Some(existing_position) => {
                        let total_size = existing_position.size + new_position.size;
                        let new_entry_price = ((existing_position.entry_price
                            * existing_position.size)
                            + (new_position.entry_price * new_position.size))
                            / total_size;

                        existing_position.size = total_size;
                        existing_position.entry_price = new_entry_price;
                        Ok(())
                    }
                    None => {
                        user.margin_positions.push(new_position);
                        Ok(())
                    }
                }
            }
            None => Err("User not found"),
        }
    }
}

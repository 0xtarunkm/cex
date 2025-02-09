use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::models::{Depth, OrderDetails};
use crate::{
    constants::LIQUIDATION_THRESHOLD,
    models::{
        CreateOrderPayload, GetQuoteResponse, MarginPosition, MarginSide, Order, OrderSide,
        OrderType, StatusCode, User,
    },
};

pub struct Orderbook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub base_asset: String,
    pub quote_asset: String,
    pub volatility: Decimal,
}

impl Orderbook {
    pub fn _new(base_asset: String, quote_asset: String) -> Self {
        Orderbook {
            bids: Vec::new(),
            asks: Vec::new(),
            base_asset,
            quote_asset,
            volatility: Decimal::from(0),
        }
    }

    pub fn fill_orders(
        &mut self,
        order: &CreateOrderPayload,
        users: &mut Arc<Mutex<Vec<User>>>,
    ) -> Decimal {
        let mut remaining_qty = order.quantity;

        match order.side {
            OrderSide::Buy => {
                let mut i = 0;

                while i < self.asks.len() && remaining_qty > Decimal::ZERO {
                    if self.asks[i].price > order.price {
                        break;
                    }

                    let fill_qty = remaining_qty.min(self.asks[i].quantity);
                    let price = self.asks[i].price;

                    if matches!(
                        order.order_type,
                        OrderType::MarginLong | OrderType::MarginShort
                    ) {
                        self.check_and_close_position(
                            order.user_id.as_str(),
                            MarginSide::Short,
                            fill_qty,
                            price,
                            users,
                        );
                    }

                    self.unlock_balance(&self.asks[i].user_id, "SOL", fill_qty, users);
                    self.unlock_balance(order.user_id.as_str(), "USDC", fill_qty * price, users);

                    self.flip_balance_update_margin_position(
                        &self.asks[i].user_id,
                        order.user_id.as_str(),
                        price,
                        fill_qty,
                        &order.order_type,
                        order.leverage,
                        users,
                    );

                    if self.asks[i].quantity > fill_qty {
                        self.asks[i].quantity -= fill_qty;
                        remaining_qty = Decimal::ZERO;
                    } else {
                        remaining_qty -= self.asks[i].quantity;
                        self.asks.remove(i);
                        continue;
                    }
                    i += 1;
                }
            }
            OrderSide::Sell => {
                let mut i = 0;

                while i < self.bids.len() && remaining_qty > Decimal::ZERO {
                    if self.bids[i].price < order.price {
                        break;
                    }

                    let fill_qty = remaining_qty.min(self.bids[i].quantity);
                    let price = self.bids[i].price;

                    if matches!(
                        order.order_type,
                        OrderType::MarginLong | OrderType::MarginShort
                    ) {
                        self.check_and_close_position(
                            order.user_id.as_str(),
                            MarginSide::Long,
                            fill_qty,
                            price,
                            users,
                        );
                    }

                    self.unlock_balance(&self.bids[i].user_id, "USDC", fill_qty * price, users);
                    self.unlock_balance(order.user_id.as_str(), "SOL", fill_qty, users);

                    self.flip_balance_update_margin_position(
                        order.user_id.as_str(),
                        &self.bids[i].user_id,
                        price,
                        fill_qty,
                        &order.order_type,
                        order.leverage,
                        users,
                    );

                    if self.bids[i].quantity > fill_qty {
                        self.bids[i].quantity -= fill_qty;
                        remaining_qty = Decimal::ZERO;
                    } else {
                        remaining_qty -= self.bids[i].quantity;
                        self.bids.remove(i);
                        continue;
                    }
                    i += 1;
                }
            }
        }

        remaining_qty
    }

    pub fn find_and_cancel_order(
        &mut self,
        user_id: &str,
        users: &mut Arc<Mutex<Vec<User>>>,
    ) -> StatusCode {
        let mut cancelled = false;

        if let Some(pos) = self.bids.iter().position(|o| o.user_id == user_id) {
            let order = self.bids.remove(pos);
            self.unlock_balance(user_id, "USDC", order.quantity * order.price, users);
            cancelled = true;
        }

        if !cancelled {
            if let Some(pos) = self.asks.iter().position(|o| o.user_id == user_id) {
                let order = self.asks.remove(pos);
                self.unlock_balance(user_id, "SOL", order.quantity, users);
                cancelled = true;
            }
        }

        if cancelled {
            StatusCode::OK
        } else {
            StatusCode::NotFound
        }
    }

    pub fn check_and_close_position(
        &self,
        user_id: &str,
        position_side: MarginSide,
        quantity: Decimal,
        price: Decimal,
        users: &mut Arc<Mutex<Vec<User>>>,
    ) {
        let mut users_guard = users.lock().unwrap();
        if let Some(user) = users_guard.iter_mut().find(|u| u.id == user_id) {
            if let Some(position) = user
                .margin_positions
                .iter_mut()
                .find(|p| p.ticker == "SOL" && p.side == position_side)
            {
                let position_data = (
                    position.entry_price,
                    position.leverage,
                    position.side.clone(),
                );
                drop(users_guard);
                let _ = self.close_margin_position(user_id, price, quantity, position_data, users);

                let mut users_guard = users.lock().unwrap();
                if let Some(user) = users_guard.iter_mut().find(|u| u.id == user_id) {
                    user.margin_positions.retain(|p| {
                        !(p.ticker == "SOL" && p.side == position_side && p.size == Decimal::ZERO)
                    });
                }
            }
        }
    }

    fn close_margin_position(
        &self,
        user_id: &str,
        close_price: Decimal,
        close_quantity: Decimal,
        position_data: (Decimal, Decimal, MarginSide),
        users: &mut Arc<Mutex<Vec<User>>>,
    ) -> Decimal {
        let mut users_guard = users.lock().unwrap();
        let user = users_guard
            .iter_mut()
            .find(|u| u.id == user_id.to_string())
            .unwrap();

        let (entry_price, leverage, side) = position_data;

        let realized_pnl = match side {
            MarginSide::Long => (close_price - entry_price) * close_quantity * leverage,
            MarginSide::Short => (entry_price - close_price) * close_quantity * leverage,
        };

        user.realized_pnl += realized_pnl;
        let position = user
            .margin_positions
            .iter_mut()
            .find(|p| p.ticker == "SOL" && p.side == side)
            .unwrap();
        position.size -= close_quantity;

        if position.size == Decimal::ZERO {
            user.margin_used -= (entry_price * close_quantity) / leverage;
        }

        if let Some(usdc_balance) = user.balances.iter_mut().find(|b| b.ticker == "USDC") {
            usdc_balance.balance += realized_pnl;
        }

        realized_pnl
    }

    fn unlock_balance(
        &self,
        user_id: &str,
        ticker: &str,
        amount: Decimal,
        users: &mut Arc<Mutex<Vec<User>>>,
    ) {
        let mut users_guard = users.lock().unwrap();
        if let Some(user) = users_guard.iter_mut().find(|u| u.id == user_id) {
            if let Some(balance) = user.balances.iter_mut().find(|b| b.ticker == ticker) {
                balance.locked_balance = balance.locked_balance.saturating_sub(amount);
            }
        }
    }

    pub fn flip_balance_update_margin_position(
        &self,
        seller_id: &str,
        buyer_id: &str,
        price: Decimal,
        quantity: Decimal,
        order_type: &OrderType,
        leverage: Option<Decimal>,
        users: &mut Arc<Mutex<Vec<User>>>,
    ) {
        let mut users_guard = users.lock().unwrap();
        let trade_value = price * quantity;

        println!("DEBUG: Trade Details:");
        println!("Price: {}, Quantity: {}, Trade Value: {}", price, quantity, trade_value);
        println!("Order Type: {:?}, Leverage: {:?}", order_type, leverage);

        // First, clear any existing locked balances for both parties
        if let Some(buyer) = users_guard.iter_mut().find(|u| u.id == buyer_id) {
            if let Some(usdc_balance) = buyer.balances.iter_mut().find(|b| b.ticker == "USDC") {
                usdc_balance.locked_balance = Decimal::ZERO;  // Reset locked balance
            }
        }

        // Update Seller balances
        if let Some(seller) = users_guard.iter_mut().find(|u| u.id == seller_id) {
            if let Some(sol_balance) = seller.balances.iter_mut().find(|b| b.ticker == "SOL") {
                sol_balance.balance -= quantity;
            }
            if let Some(usdc_balance) = seller.balances.iter_mut().find(|b| b.ticker == "USDC") {
                usdc_balance.balance += trade_value;
            }
        }

        // Update Buyer balances
        if let Some(buyer) = users_guard.iter_mut().find(|u| u.id == buyer_id) {
            match order_type {
                OrderType::MarginLong => {
                    let margin_requirement = trade_value / leverage.unwrap_or(Decimal::ONE);
                    
                    // Add SOL to buyer
                    if let Some(sol_balance) = buyer.balances.iter_mut().find(|b| b.ticker == "SOL") {
                        sol_balance.balance += quantity;
                    }
                    // Set the correct margin lock
                    if let Some(usdc_balance) = buyer.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        usdc_balance.locked_balance = margin_requirement;
                    }

                    // Create/update margin position
                    buyer.margin_positions.push(MarginPosition {
                        ticker: "SOL".to_string(),
                        side: MarginSide::Long,
                        size: quantity,
                        entry_price: price,
                        leverage: leverage.unwrap_or(Decimal::ONE),
                        liquidation_price: self.calculate_liquidation_price(
                            price,
                            leverage.unwrap_or(Decimal::ONE),
                            MarginSide::Long,
                        ),
                        unrealized_pnl: Decimal::from(0),
                    });
                },
                _ => {
                    // Handle spot orders
                    if let Some(sol_balance) = buyer.balances.iter_mut().find(|b| b.ticker == "SOL") {
                        sol_balance.balance += quantity;
                    }
                    if let Some(usdc_balance) = buyer.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        usdc_balance.balance -= trade_value;
                    }
                }
            }
        }
    }

    fn calculate_liquidation_price(
        &self,
        entry_price: Decimal,
        leverage: Decimal,
        side: MarginSide,
    ) -> Decimal {
        match side {
            MarginSide::Long => entry_price * (dec!(1) - LIQUIDATION_THRESHOLD / leverage),
            MarginSide::Short => entry_price * (dec!(1) + LIQUIDATION_THRESHOLD / leverage),
        }
    }

    fn compute_dynamic_margin(
        &self,
        price: Decimal,
        quantity: Decimal,
        leverage: Decimal,
    ) -> Decimal {
        let base_margin = (price * quantity) / leverage;

        base_margin * (Decimal::ONE + self.volatility)
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

    pub fn get_depth(&self) -> Depth {
        let mut depth: Depth = Depth {
            orders: HashMap::new(),
            market: format!("{}_{}", self.base_asset, self.quote_asset),
        };

        let mut sorted_bids: Vec<_> = self.bids.iter().collect();
        sorted_bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());

        let mut sorted_asks: Vec<_> = self.asks.iter().collect();
        sorted_asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

        for bid in sorted_bids {
            if depth.orders.contains_key(&bid.price) {
                depth.orders.get_mut(&bid.price).unwrap().quantity += bid.quantity;
            } else {
                depth.orders.insert(
                    bid.price,
                    OrderDetails {
                        type_: OrderSide::Buy,
                        quantity: bid.quantity,
                    },
                );
            }
        }

        for ask in sorted_asks {
            if depth.orders.contains_key(&ask.price) {
                depth.orders.get_mut(&ask.price).unwrap().quantity += ask.quantity;
            } else {
                depth.orders.insert(
                    ask.price,
                    OrderDetails {
                        type_: OrderSide::Sell,
                        quantity: ask.quantity,
                    },
                );
            }
        }

        depth
    }
}

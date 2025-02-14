use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tokio::sync::Mutex;

use crate::models::{
    CreateOrderPayload, Depth, GetQuoteResponse, MarginOrder, MarginPosition, OrderDetails,
    OrderSide, OrderType, User,
};
use crate::services::price_service::PriceInfo;

#[allow(dead_code)]
pub struct MarginOrderbook {
    pub longs: Vec<MarginOrder>,
    pub shorts: Vec<MarginOrder>,
    pub base_asset: String,
    pub quote_asset: String,
}

impl MarginOrderbook {
    pub fn new(base_asset: String, quote_asset: String) -> Self {
        MarginOrderbook {
            longs: Vec::new(),
            shorts: Vec::new(),
            base_asset,
            quote_asset,
        }
    }

    pub async fn fill_orders(
        &mut self,
        order: &CreateOrderPayload,
        users: &mut Arc<Mutex<Vec<User>>>,
        base_asset: &str,
    ) -> Decimal {
        let mut remaining_qty = order.quantity;

        match order.order_type {
            OrderType::MarginShort => {
                // First check and remove existing position
                {
                    let mut users_guard = users.lock().await;
                    if let Some(user) = users_guard.iter_mut().find(|u| u.id == order.user_id) {
                        if let Some(pos_idx) = user.margin_positions.iter().position(|p| 
                            p.asset == base_asset && p.position_type == OrderType::MarginLong
                        ) {
                            user.margin_positions.remove(pos_idx);
                        }
                    }
                } // users_guard is dropped here
                
                // Continue with normal order matching
                let mut i = 0;
                while i < self.longs.len() {
                    if remaining_qty <= Decimal::ZERO {
                        break;
                    }

                    let long = &self.longs[i];
                    if long.price < order.price {
                        break;
                    }

                    let fill_qty = remaining_qty.min(long.quantity);

                    self.update_margin_positions(
                        &long.user_id,
                        &order.user_id,
                        long.price,
                        fill_qty,
                        users,
                        base_asset,
                        order.leverage.unwrap_or(dec!(1)),
                    )
                    .await;

                    remaining_qty -= fill_qty;
                    self.longs[i].quantity -= fill_qty;

                    if self.longs[i].quantity == Decimal::ZERO {
                        self.longs.remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
            OrderType::MarginLong => {
                let mut i = 0;
                while i < self.shorts.len() {
                    if remaining_qty <= Decimal::ZERO {
                        break;
                    }

                    let short = &self.shorts[i];
                    if short.price > order.price {
                        break;
                    }

                    let fill_qty = remaining_qty.min(short.quantity);

                    self.update_margin_positions(
                        &order.user_id,
                        &short.user_id,
                        short.price,
                        fill_qty,
                        users,
                        base_asset,
                        order.leverage.unwrap_or(dec!(1)),
                    )
                    .await;

                    remaining_qty -= fill_qty;
                    self.shorts[i].quantity -= fill_qty;

                    if self.shorts[i].quantity == Decimal::ZERO {
                        self.shorts.remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
            OrderType::Spot => {}
        }

        remaining_qty
    }

    async fn update_margin_positions(
        &self,
        long_user_id: &str,
        short_user_id: &str,
        price: Decimal,
        quantity: Decimal,
        users: &mut Arc<Mutex<Vec<User>>>,
        base_asset: &str,
        leverage: Decimal,
    ) {
        let mut users_guard = users.lock().await;
        let trade_value = price * quantity;

        let long_margin = trade_value / leverage;
        let short_margin = (trade_value / leverage) * dec!(1.1);

        println!("DEBUG: Processing position update for user {} with quantity {}", long_user_id, quantity);

        // Handle long user position
        if let Some(long_user) = users_guard.iter_mut().find(|u| u.id == long_user_id) {
            // Check for existing short position first
            if let Some(pos_idx) = long_user
                .margin_positions
                .iter()
                .position(|p| p.asset == base_asset && p.position_type == OrderType::MarginShort)
            {
                println!("DEBUG: Found existing short position for netting");
                let short_qty = long_user.margin_positions[pos_idx].quantity;
                
                if short_qty == quantity {
                    // Exact match - remove position completely
                    println!("DEBUG: Removing fully netted position");
                    long_user.margin_positions.retain(|p| 
                        !(p.asset == base_asset && p.position_type == OrderType::MarginShort)
                    );
                    
                    // Return margin
                    if let Some(usdc_balance) = long_user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        usdc_balance.balance += short_margin;
                        usdc_balance.locked_balance -= short_margin;
                    }
                } else if short_qty > quantity {
                    // Reduce short position
                    println!("DEBUG: Reducing short position");
                    long_user.margin_positions[pos_idx].quantity -= quantity;
                    
                    // Return partial margin
                    if let Some(usdc_balance) = long_user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        let returned_margin = short_margin * (quantity / short_qty);
                        usdc_balance.balance += returned_margin;
                        usdc_balance.locked_balance -= returned_margin;
                    }
                } else {
                    // Remove short position and create smaller long position
                    println!("DEBUG: Converting to long position");
                    let short_qty = short_qty;  // Clone the quantity first
                    long_user.margin_positions.retain(|p| 
                        !(p.asset == base_asset && p.position_type == OrderType::MarginShort)
                    );
                    let new_quantity = quantity - short_qty;  // Use cloned quantity
                    
                    let mut new_position = MarginPosition {
                        asset: base_asset.to_string(),
                        quantity: new_quantity,
                        avg_price: price,
                        position_type: OrderType::MarginLong,
                        unrealized_pnl: None,
                        liquidation_price: None,
                    };
                    new_position.calculate_liquidation_price(leverage);
                    long_user.margin_positions.push(new_position);
                    
                    // Return margin and lock new amount
                    if let Some(usdc_balance) = long_user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        usdc_balance.balance += short_margin;
                        usdc_balance.locked_balance = long_margin * (new_quantity / quantity);
                    }
                }
            } else {
                // Create or update long position
                println!("DEBUG: Creating/updating long position");
                if let Some(pos) = long_user
                    .margin_positions
                    .iter_mut()
                    .find(|p| p.asset == base_asset && p.position_type == OrderType::MarginLong)
                {
                    let old_quantity = pos.quantity;
                    pos.quantity += quantity;
                    pos.avg_price = ((pos.avg_price * old_quantity) + (price * quantity)) / pos.quantity;
                    pos.calculate_liquidation_price(leverage);
                } else {
                    let mut new_position = MarginPosition {
                        asset: base_asset.to_string(),
                        quantity,
                        avg_price: price,
                        position_type: OrderType::MarginLong,
                        unrealized_pnl: None,
                        liquidation_price: None,
                    };
                    new_position.calculate_liquidation_price(leverage);
                    long_user.margin_positions.push(new_position);
                }
                
                // Lock margin
                if let Some(usdc_balance) = long_user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                    usdc_balance.balance -= long_margin;
                    usdc_balance.locked_balance += long_margin;
                }
            }
        }

        // Similar logic for short user
        if let Some(short_user) = users_guard.iter_mut().find(|u| u.id == short_user_id) {
            println!("DEBUG: Processing short user {}", short_user_id);
            // Check for existing long position first
            if let Some(pos_idx) = short_user
                .margin_positions
                .iter()
                .position(|p| p.asset == base_asset && p.position_type == OrderType::MarginLong)
            {
                let long_pos = &mut short_user.margin_positions[pos_idx];
                
                // Net the positions
                if long_pos.quantity == quantity {
                    let long_qty = long_pos.quantity;  // Clone the quantity first
                    short_user.margin_positions.remove(pos_idx);
                    let new_quantity = quantity - long_qty;  // Use cloned quantity
                    
                    let mut new_position = MarginPosition {
                        asset: base_asset.to_string(),
                        quantity: new_quantity,
                        avg_price: price,
                        position_type: OrderType::MarginShort,
                        unrealized_pnl: None,
                        liquidation_price: None,
                    };
                    new_position.calculate_liquidation_price(leverage);
                    short_user.margin_positions.push(new_position);
                    
                    if let Some(usdc_balance) = short_user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        usdc_balance.balance += long_margin;
                        usdc_balance.locked_balance = short_margin * (new_quantity / quantity);
                    }
                } else if long_pos.quantity > quantity {
                    long_pos.quantity -= quantity;
                    if let Some(usdc_balance) = short_user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        let returned_margin = long_margin * (quantity / long_pos.quantity);
                        usdc_balance.balance += returned_margin;
                        usdc_balance.locked_balance -= returned_margin;
                    }
                } else {
                    let long_qty = long_pos.quantity;  // Clone the quantity first
                    short_user.margin_positions.remove(pos_idx);
                    let new_quantity = quantity - long_qty;  // Use cloned quantity
                    
                    let mut new_position = MarginPosition {
                        asset: base_asset.to_string(),
                        quantity: new_quantity,
                        avg_price: price,
                        position_type: OrderType::MarginShort,
                        unrealized_pnl: None,
                        liquidation_price: None,
                    };
                    new_position.calculate_liquidation_price(leverage);
                    short_user.margin_positions.push(new_position);
                    
                    if let Some(usdc_balance) = short_user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        usdc_balance.balance += long_margin;
                        usdc_balance.locked_balance = short_margin * (new_quantity / quantity);
                    }
                }
            } else {
                // Create or update short position
                if let Some(pos) = short_user
                    .margin_positions
                    .iter_mut()
                    .find(|p| p.asset == base_asset && p.position_type == OrderType::MarginShort)
                {
                    let old_quantity = pos.quantity;
                    pos.quantity += quantity;
                    pos.avg_price = ((pos.avg_price * old_quantity) + (price * quantity)) / pos.quantity;
                    pos.calculate_liquidation_price(leverage);
                } else {
                    let mut new_position = MarginPosition {
                        asset: base_asset.to_string(),
                        quantity,
                        avg_price: price,
                        position_type: OrderType::MarginShort,
                        unrealized_pnl: None,
                        liquidation_price: None,
                    };
                    new_position.calculate_liquidation_price(leverage);
                    short_user.margin_positions.push(new_position);
                }
                
                // Lock margin
                if let Some(usdc_balance) = short_user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                    usdc_balance.balance -= short_margin;
                    usdc_balance.locked_balance += short_margin;
                }
            }
        }
        
        println!("DEBUG: Position update complete");
    }

    pub fn get_quote_detail(&self, quantity: Decimal, order_type: OrderType) -> GetQuoteResponse {
        let mut remaining_qty = quantity;
        let mut total_cost = Decimal::from(0);
        let mut weighted_avg_price = Decimal::from(0);

        match order_type {
            OrderType::MarginLong => {
                for short in self.shorts.iter() {
                    if remaining_qty == Decimal::ZERO {
                        break;
                    }

                    if remaining_qty > short.quantity {
                        total_cost += short.price * short.quantity;
                        remaining_qty -= short.quantity;
                    } else {
                        total_cost += short.price * remaining_qty;
                        remaining_qty = Decimal::ZERO;
                        break;
                    }
                }
            }
            OrderType::MarginShort => {
                for long in self.longs.iter() {
                    if remaining_qty == Decimal::ZERO {
                        break;
                    }
                    if remaining_qty >= long.quantity {
                        total_cost += long.price * long.quantity;
                        remaining_qty -= long.quantity;
                    } else {
                        total_cost += long.price * remaining_qty;
                        remaining_qty = Decimal::ZERO;
                        break;
                    }
                }
            }
            _ => {}
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
        let mut depth: HashMap<Decimal, OrderDetails> = HashMap::new();

        for long in &self.longs {
            depth
                .entry(long.price)
                .and_modify(|details| {
                    details.quantity += long.quantity;
                })
                .or_insert(OrderDetails {
                    type_: OrderSide::Buy,
                    quantity: long.quantity,
                });
        }

        for short in &self.shorts {
            depth
                .entry(short.price)
                .and_modify(|details| {
                    details.quantity += short.quantity;
                })
                .or_insert(OrderDetails {
                    type_: OrderSide::Sell,
                    quantity: short.quantity,
                });
        }

        Depth { orders: depth }
    }

    pub async fn get_price_info(&self) -> Option<PriceInfo> {
        let now = Utc::now().timestamp();
        if self.longs.is_empty() && self.shorts.is_empty() {
            return Some(PriceInfo {
                last_trade_price: None,
                mark_price: dec!(100),
                index_price: None,
                timestamp: now,
            });
        }
        let best_long = self.longs.first().map(|o| o.price);
        let best_short = self.shorts.first().map(|o| o.price);
        let mark_price = match (best_long, best_short) {
            (Some(l), Some(s)) => (l + s) / dec!(2),
            (Some(l), None) => l,
            (None, Some(s)) => s,
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

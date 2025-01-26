use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::{get_redis_manager, RedisManager};

use super::{Fill, Order, OrderExecutionResult, OrderSide, Orderbook, OrderbookSnapshot};

pub const BASE_CURRENCY: &str = "USDC";

// User Balance Struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub available: HashMap<String, f64>, // Available balances per asset
    pub reserved: HashMap<String, f64>,  // Reserved balances per asset
}

impl UserBalance {
    pub fn new() -> Self {
        UserBalance {
            available: HashMap::new(),
            reserved: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, asset: &str, amount: f64) {
        *self.available.entry(asset.to_string()).or_insert(0.0) += amount;
    }

    pub fn reserve(&mut self, asset: &str, amount: f64) -> bool {
        let available_balance = self.available.entry(asset.to_string()).or_insert(0.0);
        if *available_balance >= amount {
            *available_balance -= amount;
            *self.reserved.entry(asset.to_string()).or_insert(0.0) += amount;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self, asset: &str, amount: f64) {
        let reserved_balance = self.reserved.entry(asset.to_string()).or_insert(0.0);
        if *reserved_balance >= amount {
            *reserved_balance -= amount;
            *self.available.entry(asset.to_string()).or_insert(0.0) += amount;
        }
    }

    pub fn settle(&mut self, asset: &str, amount: f64) {
        let reserved_balance = self.reserved.entry(asset.to_string()).or_insert(0.0);
        if *reserved_balance >= amount {
            *reserved_balance -= amount;
        }
    }
}

// Engine Struct
#[derive(Debug)]
pub struct Engine {
    orderbooks: HashMap<String, Orderbook>, // Key: market ticker (e.g., "BTC_USDC")
    balances: HashMap<String, UserBalance>, // Key: user_id
}

impl Engine {
    pub fn new() -> Self {
        let mut orderbooks = HashMap::new();
        orderbooks.insert(
            "BTC_USDC".to_string(),
            Orderbook::new("BTC".to_string(), vec![], vec![], 0, 0.0),
        );
        let mut balances = HashMap::new();
        balances.insert(
            "1".to_string(),
            UserBalance {
                available: HashMap::from([(BASE_CURRENCY.to_string(), 10_000_000.0)]),
                reserved: HashMap::new(),
            },
        );

        let engine = Engine {
            orderbooks,
            balances,
        };

        engine
    }

    pub fn process(&mut self, message: MessageFromApi, client_id: String) {
        match message {
            MessageFromApi::CreateOrder {
                market,
                price,
                quantity,
                side,
                user_id,
            } => {
                if let Ok((executed_qty, fills, order_id)) =
                    self.create_order(&market, &price, &quantity, side, &user_id)
                {
                    let redis_manager = get_redis_manager().lock().unwrap();
                    println!("called");
                    redis_manager
                        .send_to_api(order_id, executed_qty, client_id)
                        .unwrap();
                } else {
                    // Send ORDER_CANCELLED message
                    println!("Order cancelled");
                }
            }
            MessageFromApi::CancelOrder { order_id, market } => {
                if let Some(orderbook) = self.orderbooks.get_mut(&market) {
                    if let Some(order) = orderbook
                        .asks
                        .iter()
                        .find(|o| o.order_id == order_id)
                        .cloned()
                    {
                        let price = orderbook.cancel_ask(&order_id);
                        if let Some(price) = price {
                            let left_quantity = (order.quantity - order.filled) * order.price;
                            if let Some(balance) = self.balances.get_mut(&order.user_id) {
                                if let Some(quote_balance) =
                                    balance.available.get_mut(BASE_CURRENCY)
                                {
                                    *quote_balance += left_quantity;
                                }
                                if let Some(reserved_balance) =
                                    balance.reserved.get_mut(BASE_CURRENCY)
                                {
                                    *reserved_balance -= left_quantity;
                                }
                            }
                            self.send_updated_depth_at(&price.to_string(), &market);
                        }
                    }
                }
            }
            MessageFromApi::GetOpenOrders { market, user_id } => {
                if let Some(orderbook) = self.orderbooks.get(&market) {
                    let open_orders = orderbook.get_open_orders(&user_id);
                    println!("Open orders: {:?}", open_orders);
                }
            }
            MessageFromApi::OnRamp { user_id, amount } => {
                self.on_ramp(&user_id, amount);
            }
            MessageFromApi::GetDepth { market } => {
                if let Some(orderbook) = self.orderbooks.get(&market) {
                    let depth = orderbook.get_depth();
                    println!("Market depth: {:?}", depth);
                }
            }
        }
    }

    pub fn create_order(
        &mut self,
        market: &str,
        price: &str,
        quantity: &str,
        side: OrderSide,
        user_id: &str,
    ) -> Result<(f64, Vec<Fill>, String), String> {
        let base_asset = market.split('_').next().unwrap();
        let quote_asset = market.split('_').nth(1).unwrap();

        self.check_and_lock_funds(base_asset, quote_asset, &side, user_id, price, quantity)?;

        let orderbook = self
            .orderbooks
            .get_mut(market)
            .ok_or("No orderbook found")?;

        let order = Order {
            price: price.parse().unwrap(),
            quantity: quantity.parse().unwrap(),
            order_id: format!("{:x}", rand::random::<u64>()),
            filled: 0.0,
            side: side.clone(),
            user_id: user_id.to_string(),
        };

        let OrderExecutionResult {
            executed_qty,
            fills,
        } = orderbook.add_order(order.clone());

        self.update_balance(
            user_id,
            base_asset,
            quote_asset,
            &side,
            &fills,
            executed_qty,
        );

        Ok((executed_qty, fills, order.order_id))
    }

    fn check_and_lock_funds(
        &mut self,
        base_asset: &str,
        quote_asset: &str,
        side: &OrderSide,
        user_id: &str,
        price: &str,
        quantity: &str,
    ) -> Result<(), String> {
        let price: f64 = price.parse().unwrap();
        let quantity: f64 = quantity.parse().unwrap();

        let user_balance = self.balances.get_mut(user_id).ok_or("User not found")?;

        match side {
            OrderSide::Buy => {
                let quote_balance = user_balance
                    .available
                    .get_mut(quote_asset)
                    .ok_or("Quote asset not found")?;
                if *quote_balance < price * quantity {
                    return Err("Insufficient funds".to_string());
                }
                *quote_balance -= price * quantity;
                *user_balance
                    .reserved
                    .entry(quote_asset.to_string())
                    .or_insert(0.0) += price * quantity;
            }
            OrderSide::Sell => {
                let base_balance = user_balance
                    .available
                    .get_mut(base_asset)
                    .ok_or("Base asset not found")?;
                if *base_balance < quantity {
                    return Err("Insufficient funds".to_string());
                }
                *base_balance -= quantity;
                *user_balance
                    .reserved
                    .entry(base_asset.to_string())
                    .or_insert(0.0) += quantity;
            }
        }

        Ok(())
    }

    fn update_balance(
        &mut self,
        user_id: &str,
        base_asset: &str,
        quote_asset: &str,
        side: &OrderSide,
        fills: &[Fill],
        executed_qty: f64,
    ) {
        for fill in fills {
            let counterparty_id = &fill.other_user_id;
            let mut counterparty_balance = self
                .balances
                .entry(counterparty_id.clone())
                .or_insert_with(UserBalance::new);

            let fill_price = fill.price.parse::<f64>().unwrap();

            match side {
                OrderSide::Buy => {
                    counterparty_balance
                        .available
                        .entry(quote_asset.to_string())
                        .and_modify(|b| *b += fill.qty * fill_price);
                    counterparty_balance
                        .reserved
                        .entry(base_asset.to_string())
                        .and_modify(|b| *b -= fill.qty);
                }
                OrderSide::Sell => {
                    counterparty_balance
                        .available
                        .entry(base_asset.to_string())
                        .and_modify(|b| *b += fill.qty);
                    counterparty_balance
                        .reserved
                        .entry(quote_asset.to_string())
                        .and_modify(|b| *b -= fill.qty * fill_price);
                }
            }
        }
    }

    fn send_updated_depth_at(&self, price: &str, market: &str) {
        if let Some(orderbook) = self.orderbooks.get(market) {
            let depth = orderbook.get_depth();
            let updated_bids = depth
                .bids
                .iter()
                .filter(|x| x.0 == price)
                .cloned()
                .collect::<Vec<_>>();
            let updated_asks = depth
                .asks
                .iter()
                .filter(|x| x.0 == price)
                .cloned()
                .collect::<Vec<_>>();

            println!(
                "Updated Depth at {}: Bids: {:?}, Asks: {:?}",
                price, updated_bids, updated_asks
            );
        }
    }

    pub fn on_ramp(&mut self, user_id: &str, amount: f64) {
        let user_balance = self
            .balances
            .entry(user_id.to_string())
            .or_insert_with(UserBalance::new);
        user_balance.deposit(BASE_CURRENCY, amount);
    }
}

// Snapshot Struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub orderbooks: Vec<OrderbookSnapshot>,
    pub balances: HashMap<String, UserBalance>,
}

// MessageFromApi Enum
#[derive(Debug, Deserialize)]
pub enum MessageFromApi {
    CreateOrder {
        market: String,
        price: String,
        quantity: String,
        side: OrderSide,
        user_id: String,
    },
    CancelOrder {
        order_id: String,
        market: String,
    },
    GetOpenOrders {
        market: String,
        user_id: String,
    },
    OnRamp {
        user_id: String,
        amount: f64,
    },
    GetDepth {
        market: String,
    },
}

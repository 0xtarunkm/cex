use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    models::{
        AssetBalance, Fill, MessageFromApi, MessageToApi, OpenOrdersPayload, Order,
        OrderCancelledPayload, OrderPlacedPayload, OrderSide,
    },
    utils::redis_manager::RedisManager,
};

use super::Orderbook;

type UserBalances = HashMap<String, AssetBalance>;

pub struct Engine {
    orderbooks: Arc<Mutex<Vec<Orderbook>>>,
    balances: Arc<Mutex<HashMap<String, UserBalances>>>,
}

impl Engine {
    pub fn new() -> Self {
        let mut orderbook = Vec::new();
        orderbook.push(Orderbook {
            bids: Vec::new(),
            asks: Vec::new(),
            base_asset: "SOL".to_string(),
            quote_asset: "USDC".to_string(),
            last_trade_id: 1,
        });

        let mut balances = HashMap::new();
        balances.insert("1".to_string(), {
            let mut user_balances = HashMap::new();
            user_balances.insert("USDC".to_string(), AssetBalance {
                available: Decimal::from(10000),
                locked: Decimal::from(0),
            });
            user_balances.insert("SOL".to_string(), AssetBalance {
                available: Decimal::from(1000),
                locked: Decimal::from(0),
            });
            user_balances
        });
        balances.insert("2".to_string(), {
            let mut user_balances = HashMap::new();
            user_balances.insert("USDC".to_string(), AssetBalance {
                available: Decimal::from(10000),
                locked: Decimal::from(0),
            });
            user_balances.insert("SOL".to_string(), AssetBalance {
                available: Decimal::from(1000),
                locked: Decimal::from(0),
            });
            user_balances
        });

        Engine {
            orderbooks: Arc::new(Mutex::new(orderbook)),
            balances: Arc::new(Mutex::new(balances)),
        }
    }

    pub fn process(&mut self, client_id: String, message: MessageFromApi) {
        match message {
            MessageFromApi::CreateOrder { data } => {
                let result = self.create_order(
                    &data.market,
                    &data.price,
                    &data.quantity,
                    &data.side,
                    &data.user_id,
                );

                match result {
                    Ok((executed_qty, fills, order_id)) => {
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderPlaced {
                            payload: OrderPlacedPayload {
                                order_id,
                                executed_qty,
                                fills,
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                    Err(e) => {
                        println!("error: {}", e);
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderCancelled {
                            payload: OrderCancelledPayload {
                                order_id: data.market,
                                executed_qty: Decimal::from(0),
                                remaining_qty: Decimal::from(0),
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                }
            }
            MessageFromApi::CancelOrder { data } => {
                let mut orderbook_guard = self.orderbooks.lock().unwrap();
                let orderbook = orderbook_guard
                    .iter_mut()
                    .find(|o| o.ticker() == data.market)
                    .ok_or("No orderbook found");

                if let Ok(orderbook) = orderbook {
                    let order_lookup = orderbook
                        .asks
                        .iter()
                        .position(|o| o.order_id == data.order_id)
                        .map(|i| ("asks", i))
                        .or_else(|| {
                            orderbook
                                .bids
                                .iter()
                                .position(|o| o.order_id == data.order_id)
                                .map(|i| ("bids", i))
                        });

                    if let Some((side_str, index)) = order_lookup {
                        let order = if side_str == "asks" {
                            orderbook.asks[index].clone()
                        } else {
                            orderbook.bids[index].clone()
                        };

                        let quote_asset = data
                            .market
                            .split('_')
                            .nth(1)
                            .unwrap_or_default()
                            .to_string();

                        match order.side {
                            OrderSide::Buy => {
                                let left_quantity = (order.quantity - order.filled) * order.price;

                                let mut balances = self.balances.lock().unwrap();
                                if let Some(user_balances) = balances.get_mut(&order.user_id) {
                                    if let Some(quote_balance) = user_balances.get_mut(&quote_asset)
                                    {
                                        quote_balance.available += left_quantity;
                                        quote_balance.locked -= left_quantity;
                                    }
                                }
                            }
                            OrderSide::Sell => {
                                let left_quantity = order.quantity - order.filled;

                                let mut balances = self.balances.lock().unwrap();
                                if let Some(user_balances) = balances.get_mut(&order.user_id) {
                                    if let Some(base_balance) = user_balances.get_mut(&quote_asset)
                                    {
                                        base_balance.available += left_quantity;
                                        base_balance.locked -= left_quantity;
                                    }
                                }
                            }
                        }

                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderCancelled {
                            payload: OrderCancelledPayload {
                                order_id: order.order_id.clone(),
                                executed_qty: Decimal::new(0, 0),
                                remaining_qty: Decimal::new(0, 0),
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    } else {
                        println!("No order found with id: {}", data.order_id);
                    }
                } else {
                    println!("orderbook not found for market: {}", data.market);
                }
            }
            MessageFromApi::GetOpenOrders { data } => {
                let mut orderbook_guard = self.orderbooks.lock().unwrap();
                let orderbook = orderbook_guard
                    .iter_mut()
                    .find(|o| o.ticker() == data.market)
                    .ok_or("No orderbook found");
                if let Ok(orderbook) = orderbook {
                    let open_orders = orderbook.get_open_orders(data.user_id);

                    let redis_manager = RedisManager::instance();
                    let message = MessageToApi::OpenOrders {
                        payload: OpenOrdersPayload { open_orders },
                    };

                    let _ = redis_manager.send_to_api(&client_id, &message);
                } else {
                    println!("no orderbook found");
                }
            }
            MessageFromApi::GetDepth { data } => {
                let mut orderbook_guard = self.orderbooks.lock().unwrap();
                let orderbook = orderbook_guard
                    .iter_mut()
                    .find(|o| o.ticker() == data.market)
                    .ok_or("No orderbook found");
                if let Ok(orderbook) = orderbook {
                    let depth = orderbook.get_depth();

                    let redis_manager = RedisManager::instance();
                    let message = MessageToApi::Depth { payload: depth };

                    let _ = redis_manager.send_to_api(&client_id, &message);
                } else {
                    println!("no orderbook found");
                }
            }
            MessageFromApi::OnRamp { data } => {
                self.on_ramp(&data.user_id, data.amount);
            }
        }
    }

    fn create_order(
        &mut self,
        market: &str,
        price: &Decimal,
        quantity: &Decimal,
        side: &OrderSide,
        user_id: &str,
    ) -> Result<(Decimal, Vec<Fill>, String), Box<dyn std::error::Error>> {
        let assets: Vec<&str> = market.split('_').collect();
        if assets.len() != 2 {
            return Err("Invalid market format".into());
        }

        let order_id = Uuid::new_v4().to_string();

        self.check_and_lock_funds(assets[0], assets[1], side, user_id, price, quantity)?;

        let mut orderbook_guard = self.orderbooks.lock().unwrap();
        let orderbook = orderbook_guard
            .iter_mut()
            .find(|o| o.ticker() == market)
            .ok_or("No orderbook found")?;

        let order = Order {
            price: price.clone(),
            quantity: quantity.clone(),
            filled: Decimal::from(0),
            side: side.clone(),
            user_id: user_id.to_string(),
            order_id: order_id.clone(),
        };

        let result = orderbook.add_order(order);
        let (executed_qty, fills) = (result.executed_qty, result.fills);

        drop(orderbook_guard);

        self.update_balance(user_id, assets[0], assets[1], side, &fills)?;
        self.publish_ws_trades(&fills, user_id, market);
        self.publish_ws_depth_updates(&fills, price, side, market);

        Ok((executed_qty, fills, order_id))
    }

    fn check_and_lock_funds(
        &mut self,
        base_asset: &str,
        quote_asset: &str,
        side: &OrderSide,
        user_id: &str,
        price: &Decimal,
        quantity: &Decimal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut balances = self.balances.lock().unwrap();

        let user_balances = balances.get_mut(user_id).ok_or("User not found")?;

        match side {
            OrderSide::Buy => {
                let quote_balance = user_balances
                    .get_mut(quote_asset)
                    .ok_or("Quote asset balance not found")?;

                let required_amount = price * quantity;
                if quote_balance.available < required_amount {
                    return Err("Insufficient funds".into());
                }

                quote_balance.available -= required_amount;
                quote_balance.locked += required_amount;
            }
            OrderSide::Sell => {
                let base_balance = user_balances
                    .get_mut(base_asset)
                    .ok_or("Base asset balance not found")?;

                if base_balance.available < *quantity {
                    return Err("Insufficient funds".into());
                }

                base_balance.available -= *quantity;
                base_balance.locked += *quantity;
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut balances = self.balances.lock().unwrap();

        match side {
            OrderSide::Buy => {
                for fill in fills {
                    {
                        let other_user_balances = balances
                            .get_mut(&fill.other_user_id)
                            .ok_or("Other user not found")?;
                        let other_quote_balance = other_user_balances
                            .get_mut(quote_asset)
                            .ok_or("Quote asset balance not found for other user")?;
                        other_quote_balance.available += fill.quantity * fill.price;

                        let other_base_balance = other_user_balances
                            .get_mut(base_asset)
                            .ok_or("Base asset balance not found for other user")?;
                        other_base_balance.locked -= fill.quantity;
                    }

                    {
                        let user_balances = balances.get_mut(user_id).ok_or("User not found")?;
                        let user_quote_balance = user_balances
                            .get_mut(quote_asset)
                            .ok_or("Quote asset balance not found for user")?;
                        user_quote_balance.locked -= fill.quantity * fill.price;

                        let user_base_balance = user_balances
                            .get_mut(base_asset)
                            .ok_or("Base asset balance not found for user")?;
                        user_base_balance.available += fill.quantity;
                    }
                }
            }
            OrderSide::Sell => {
                for fill in fills {
                    {
                        let other_user_balances = balances
                            .get_mut(&fill.other_user_id)
                            .ok_or("Other user not found")?;
                        let other_quote_balance = other_user_balances
                            .get_mut(quote_asset)
                            .ok_or("Quote asset balance not found for other user")?;
                        other_quote_balance.locked -= fill.quantity * fill.price;

                        let other_base_balance = other_user_balances
                            .get_mut(base_asset)
                            .ok_or("Base asset balance not found for other user")?;
                        other_base_balance.available += fill.quantity;
                    }

                    {
                        let user_balances = balances.get_mut(user_id).ok_or("User not found")?;
                        let user_quote_balance = user_balances
                            .get_mut(quote_asset)
                            .ok_or("Quote asset balance not found for user")?;
                        user_quote_balance.available += fill.quantity * fill.price;

                        let user_base_balance = user_balances
                            .get_mut(base_asset)
                            .ok_or("Base asset balance not found for user")?;
                        user_base_balance.locked -= fill.quantity;
                    }
                }
            }
        }

        Ok(())
    }

    fn publish_ws_trades(&self, fills: &[Fill], user_id: &str, market: &str) {
        let redis_manager = RedisManager::instance();

        for fill in fills {
            let trade_message = serde_json::json!({
                "stream": format!("trade@{}", market),
                "data": {
                    "e": "trade",
                    "t": fill.trade_id,
                    "m": fill.other_user_id == user_id,
                    "p": fill.price,
                    "q": fill.quantity.to_string(),
                    "s": market,
                }
            });

            let _ = redis_manager.publish_message(&format!("trade@{}", market), &trade_message);
        }
    }

    fn publish_ws_depth_updates(
        &self,
        fills: &[Fill],
        price: &Decimal,
        side: &OrderSide,
        market: &str,
    ) {
        let orderbook_guard = self.orderbooks.lock().unwrap();
        let orderbook = orderbook_guard.iter().find(|o| o.ticker() == market);

        if let Some(orderbook) = orderbook {
            let depth = orderbook.get_depth();

            let redis_manager = RedisManager::instance();
            let stream = format!("depth@{}", market);

            match side {
                OrderSide::Buy => {
                    let updated_asks: Vec<_> = depth
                        .asks
                        .iter()
                        .filter(|x| fills.iter().any(|f| f.price.to_string() == x.0.to_string()))
                        .cloned()
                        .collect();

                    let updated_bid = depth.bids.iter().find(|x| x.0 == *price).cloned();

                    let data = serde_json::json!({
                        "stream": stream,
                        "data": {
                            "a": updated_asks,
                            "b": updated_bid.map_or(Vec::new(), |b| vec![b]),
                            "e": "depth"
                        }
                    });

                    let _ = redis_manager.publish_message(&stream, &data);
                }
                OrderSide::Sell => {
                    let updated_bids: Vec<_> = depth
                        .bids
                        .iter()
                        .filter(|x| fills.iter().any(|f| f.price == x.0))
                        .cloned()
                        .collect();

                    let updated_ask = depth.asks.iter().find(|x| x.0 == *price).cloned();

                    let data = serde_json::json!({
                        "stream": stream,
                        "data": {
                            "a": updated_ask.map_or(Vec::new(), |a| vec![a]),
                            "b": updated_bids,
                            "e": "depth"
                        }
                    });

                    let _ = redis_manager.publish_message(&stream, &data);
                } 
            }
        }
    }

    pub fn on_ramp(&mut self, user_id: &String, amount: Decimal) {
        let mut balances = self.balances.lock().unwrap();
        
        if !balances.contains_key(user_id) {
            let mut user_balance = HashMap::new();
            user_balance.insert("USDC".to_string(), AssetBalance {
                available: amount,
                locked: Decimal::new(0, 0),
            });
            balances.insert(user_id.to_string(), user_balance);
        } else {
            if let Some(user_balance) = balances.get_mut(user_id) {
                user_balance.entry("USDC".to_string())
                    .or_insert(AssetBalance {
                        available: Decimal::new(0, 0),
                        locked: Decimal::new(0, 0),
                    })
                    .available += amount;
            }
        }
    }
}

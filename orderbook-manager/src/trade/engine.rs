use std::{
    collections::HashMap,
    str::FromStr,
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
        Engine {
            orderbooks: Arc::new(Mutex::new(orderbook)),
            balances: Arc::new(Mutex::new(HashMap::new())),
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
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderCancelled {
                            payload: OrderCancelledPayload {
                                order_id: String::new(),
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
                    let order = orderbook
                        .asks
                        .iter()
                        .find(|o| o.order_id == data.order_id)
                        .or_else(|| orderbook.bids.iter().find(|o| o.order_id == data.order_id));

                    match order {
                        Some(order) => {
                            let redis_manager = RedisManager::instance();
                            let message = MessageToApi::OrderCancelled {
                                payload: OrderCancelledPayload {
                                    order_id: order.order_id.clone(),
                                    executed_qty: Decimal::new(0, 0),
                                    remaining_qty: Decimal::new(0, 0),
                                },
                            };

                            let _ = redis_manager.send_to_api(&client_id, &message);
                        }
                        None => {}
                    }
                } else {
                    println!("orderbook not found");
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
            _ => {}
        }
    }

    fn create_order(
        &mut self,
        market: &str,
        price: &str,
        quantity: &str,
        side: &str,
        user_id: &str,
    ) -> Result<(Decimal, Vec<Fill>, String), Box<dyn std::error::Error>> {
        let mut orderbook_guard = self.orderbooks.lock().unwrap();
        let orderbook = orderbook_guard
            .iter_mut()
            .find(|o| o.ticker() == market)
            .ok_or("No orderbook found")?;

        let assets: Vec<&str> = market.split('_').collect();
        if assets.len() != 2 {
            return Err("Invalid market format".into());
        }

        let price = Decimal::from_str(price)?;
        let quantity = Decimal::from_str(quantity)?;
        let order_id = Uuid::new_v4().to_string();

        let side = match side.to_lowercase().as_str() {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => return Err("Invalid order side".into()),
        };

        let order = Order {
            price,
            quantity,
            filled: Decimal::from(0),
            side,
            user_id: user_id.to_string(),
            order_id: order_id.clone(),
        };

        let result = orderbook.add_order(order);

        Ok((result.executed_qty, result.fills, order_id))
    }
}

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::{
    models::{
        Balances, CancelOrderPayload, CreateOrderPayload, GetQuoteResponse, MessageFromApi, MessageToApi, OpenOrdersPayload, OrderCancelledPayload, OrderPlacedPayload, OrderSide, OrderType, SpotOrder, StatusCode, User, UserBalancesPayload
    },
    services::redis_manager::RedisManager,
};

use super::SpotOrderbook;

pub struct Engine {
    pub spot_orderbooks: Arc<Mutex<HashMap<String, Arc<Mutex<SpotOrderbook>>>>>,
    pub users: Arc<Mutex<Vec<User>>>,
}

impl Engine {
    pub fn new() -> Self {
        let mut spot_orderbooks = HashMap::new();

        let markets = vec![("SOL", "USDC"), ("BTC", "USDC"), ("ETH", "USDC")];

        for (base, quote) in markets {
            let orderbook = SpotOrderbook::new(
                base.to_string(),
                quote.to_string(),
            );
            
            spot_orderbooks.insert(
                format!("{}_{}", base, quote),
                Arc::new(Mutex::new(orderbook)),
            );
        }

        let users = vec![
            User {
                id: "1".to_string(),
                balances: vec![
                    Balances {
                        ticker: "SOL".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balances {
                        ticker: "BTC".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balances {
                        ticker: "ETH".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balances {
                        ticker: "USDC".to_string(),
                        balance: Decimal::from(10_000),
                        locked_balance: Decimal::from(0),
                    },
                ],
                margin_enabled: true,
                margin_positions: vec![],
                margin_used: Decimal::from(0),
                max_leverage: Decimal::from(10),
                realized_pnl: Decimal::from(0),
            },
            User {
                id: "2".to_string(),
                balances: vec![
                    Balances {
                        ticker: "SOL".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balances {
                        ticker: "BTC".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balances {
                        ticker: "ETH".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balances {
                        ticker: "USDC".to_string(),
                        balance: Decimal::from(10_000),
                        locked_balance: Decimal::from(0),
                    },
                ],
                margin_enabled: true,
                margin_positions: vec![],
                margin_used: Decimal::from(0),
                max_leverage: Decimal::from(10),
                realized_pnl: Decimal::from(0),
            },
        ];

        Engine {
            spot_orderbooks: Arc::new(Mutex::new(spot_orderbooks)),
            users: Arc::new(Mutex::new(users)),
        }
    }

    pub fn process(&mut self, client_id: String, message: MessageFromApi) {
        match message {
            MessageFromApi::CreateOrder { data } => {
                let result = self.create_order(&data);

                match result {
                    Ok((remaining_qty, filled_qty, order_id)) => {
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderPlaced {
                            payload: OrderPlacedPayload {
                                order_id,
                                remaining_qty,
                                filled_qty,
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                    Err(e) => {
                        println!("error: {}", e);
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderCancelled {
                            payload: OrderCancelledPayload {
                                message: Some(String::from("Order execution failed")),
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                }
            }
            MessageFromApi::CancelOrder { data } => {
                let result = self.cancel_order(&data);

                match result {
                    Ok(()) => {
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderCancelled {
                            payload: OrderCancelledPayload {
                                message: Some(String::from("ORDER CANCELLED")),
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                    Err(e) => {
                        println!("error: {}", e);
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderCancelled {
                            payload: OrderCancelledPayload {
                                message: Some(String::from("ORDER CANCELLATION FAILED")),
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                }
            }
            MessageFromApi::GetQuote { data } => {
                let orderbooks = self.spot_orderbooks.lock().unwrap();
                let orderbook = orderbooks
                    .get(&data.market)
                    .ok_or("Market not found")
                    .unwrap();

                let result = orderbook
                    .lock()
                    .unwrap()
                    .get_quote_detail(data.quantity, data.side);

                let redis_manager = RedisManager::instance();
                let message = MessageToApi::Quote {
                    payload: GetQuoteResponse {
                        avg_price: result.avg_price,
                        quantity: result.quantity,
                        total_cost: result.total_cost,
                    },
                };

                let _ = redis_manager.send_to_api(&client_id, &message);
            }
            MessageFromApi::GetDepth { data } => {
                let orderbooks = self.spot_orderbooks.lock().unwrap();
                let orderbook = orderbooks
                    .get(&data.market)
                    .ok_or("Market not found")
                    .unwrap();

                let orderbook_guard = orderbook.lock().unwrap();

                let result = orderbook_guard.get_depth();

                let redis_manager = RedisManager::instance();
                let message = MessageToApi::Depth { payload: result };

                let _ = redis_manager.send_to_api(&client_id, &message);
            }
            MessageFromApi::GetOpenOrders { data } => {
                let orderbooks = self.spot_orderbooks.lock().unwrap();
                let orderbook = orderbooks
                    .get(&data.market)
                    .ok_or("Market not found")
                    .unwrap();

                let mut open_orders = Vec::new();

                for bid in orderbook.lock().unwrap().bids.iter() {
                    if bid.user_id == data.user_id {
                        open_orders.push(bid.clone());
                    }
                }

                for ask in orderbook.lock().unwrap().asks.iter() {
                    if ask.user_id == data.user_id {
                        open_orders.push(ask.clone());
                    }
                }

                let redis_manager = RedisManager::instance();
                let message = MessageToApi::OpenOrders {
                    payload: OpenOrdersPayload { open_orders },
                };

                let _ = redis_manager.send_to_api(&client_id, &message);
            }
            MessageFromApi::GetUserBalances { data } => {
                let mut users = self.users.lock().unwrap();
                let user = users
                    .iter_mut()
                    .find(|u| u.id == data.user_id)
                    .expect("User not found");

                let redis_manager = RedisManager::instance();
                let message = MessageToApi::UserBalances {
                    payload: UserBalancesPayload {
                        balances: user.balances.clone(),
                    },
                };

                let _ = redis_manager.send_to_api(&client_id, &message);
            }
        }
    }

    pub fn create_order(
        &mut self,
        payload: &CreateOrderPayload,
    ) -> Result<(Decimal, Decimal, String), Box<dyn std::error::Error>> {
        let order_id = Uuid::new_v4().to_string();

        // validation check for spot and margin order
        match payload.order_type {
            OrderType::MarginLong | OrderType::MarginShort => {
                // if !self.validate_margin_requirements(&payload) {
                //     return Err("Insufficient margin or invalid margin requirements".into());
                // }
                Ok((Decimal::ZERO, Decimal::ZERO, order_id))
            }
            OrderType::Spot => {
                if !self.validate_spot_balance(&payload) {
                    return Err("Insufficient balance for spot trade".into());
                }

                let orderbooks = self.spot_orderbooks.lock().unwrap();
                let orderbook = orderbooks
                    .get(&payload.market)
                    .ok_or("Market not found")
                    .unwrap();

                let remaining_qty = orderbook.lock().unwrap().fill_orders(
                    payload,
                    &mut self.users,
                    &payload.market.split('_').nth(0).unwrap().to_string(),
                    &payload.market.split('_').nth(1).unwrap().to_string(),
                );

                if remaining_qty == Decimal::from(0) {
                    return Ok((Decimal::ZERO, payload.quantity, order_id.clone()));
                }

                match payload.side {
                    OrderSide::Buy => {
                        let mut orderbook_guard = orderbook.lock().unwrap();
                        orderbook_guard.bids.push(SpotOrder {
                            id: order_id.clone(),
                            user_id: payload.user_id.clone(),
                            price: payload.price,
                            quantity: remaining_qty,
                            side: payload.side.clone(),
                            timestamp: Utc::now().timestamp(),
                        });
                        // (highest price first)
                        orderbook_guard.bids.sort_by(|a, b| {
                            b.price
                                .cmp(&a.price)
                                .then_with(|| a.timestamp.cmp(&b.timestamp))
                        });
                    }
                    OrderSide::Sell => {
                        let mut orderbook_guard = orderbook.lock().unwrap();
                        orderbook_guard.asks.push(SpotOrder {
                            id: order_id.clone(),
                            user_id: payload.user_id.clone(),
                            price: payload.price,
                            quantity: remaining_qty,
                            side: payload.side.clone(),
                            timestamp: Utc::now().timestamp(),
                        });
                        // (lowest price first)
                        orderbook_guard.asks.sort_by(|a, b| {
                            a.price
                                .cmp(&b.price)
                                .then_with(|| a.timestamp.cmp(&b.timestamp))
                        });
                    }
                }

                let filled_qty = payload.quantity.checked_sub(remaining_qty).unwrap();

                Ok((remaining_qty, filled_qty, order_id))
            }
        }
    }

    pub fn cancel_order(
        &mut self,
        payload: &CancelOrderPayload,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let orderbooks = self.spot_orderbooks.lock().unwrap();
        let orderbook = orderbooks.get(&payload.market).ok_or("Market not found")?;

        let mut orderbook_guard = orderbook.lock().unwrap();

        if let Some(bid_index) = orderbook_guard
            .bids
            .iter()
            .position(|order| order.id == payload.order_id)
        {
            let order = &orderbook_guard.bids[bid_index];

            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.iter_mut().find(|u| u.id == order.user_id) {
                let quote_asset = payload.market.split('_').nth(1).unwrap();
                if let Some(balance) = user.balances.iter_mut().find(|b| b.ticker == quote_asset) {
                    let locked_amount = order.price * order.quantity;
                    balance.locked_balance -= locked_amount;
                }
            }

            let _ = orderbook_guard.bids.remove(bid_index);
            return Ok(());
        }

        if let Some(ask_index) = orderbook_guard
            .asks
            .iter()
            .position(|order| order.id == payload.order_id)
        {
            let order = &orderbook_guard.asks[ask_index];

            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.iter_mut().find(|u| u.id == order.user_id) {
                let base_asset = payload.market.split('_').nth(0).unwrap();
                if let Some(balance) = user.balances.iter_mut().find(|b| b.ticker == base_asset) {
                    balance.locked_balance -= order.quantity;
                }
            }

            let _ = &orderbook_guard.asks.remove(ask_index);
            return Ok(());
        }

        Err("Order not found".into())
    }

    //     fn validate_margin_requirements(&self, payload: &CreateOrderPayload) -> bool {
    //         let mut users = self.users.lock().unwrap();
    //         let user = users
    //             .iter_mut()
    //             .find(|u| u.id == payload.user_id)
    //             .expect("User not found");

    //         if !user.margin_enabled {
    //             return false;
    //         }

    //         let leverage = payload.leverage.unwrap_or(dec!(1));
    //         if leverage > MAX_LEVERAGE {
    //             return false;
    //         }

    //         let usdc_balance = user
    //             .balances
    //             .iter()
    //             .find(|b| b.ticker == "USDC")
    //             .map(|b| b.balance - b.locked_balance)
    //             .unwrap_or(dec!(0));

    //         let required_margin = (payload.price * payload.quantity) / leverage;

    //         match payload.order_type {
    //             OrderType::MarginLong => {
    //                 if usdc_balance >= required_margin {
    //                     if let Some(balance) = user
    //                         .balances
    //                         .iter_mut()
    //                         .find(|b| b.ticker == "USDC".to_string())
    //                     {
    //                         balance.locked_balance += required_margin;
    //                     }
    //                     return true;
    //                 } else {
    //                     return false;
    //                 }
    //             }
    //             OrderType::MarginShort => {
    //                 let safety_multiplier = dec!(1.1);
    //                 let adjusted_required_margin = required_margin * safety_multiplier;

    //                 if usdc_balance >= adjusted_required_margin {
    //                     if let Some(balance) = user.balances.iter_mut().find(|b| b.ticker == "USDC") {
    //                         balance.locked_balance += adjusted_required_margin;
    //                     }
    //                     return true;
    //                 } else {
    //                     return false;
    //                 }
    //             }
    //             OrderType::Spot => {}
    //         }
    //         true
    //     }

    fn validate_spot_balance(&self, payload: &CreateOrderPayload) -> bool {
        let mut users = self.users.lock().unwrap();
        let user = users
            .iter_mut()
            .find(|u| u.id == payload.user_id)
            .expect("User not found");

        match payload.side {
            OrderSide::Buy => {
                let usdc_balance = user
                    .balances
                    .iter()
                    .find(|b| b.ticker == "USDC".to_string())
                    .map(|b| b.balance - b.locked_balance)
                    .unwrap_or(dec!(0));
                let required_amount = payload.price * payload.quantity;
                if usdc_balance >= required_amount {
                    if let Some(balance) = user
                        .balances
                        .iter_mut()
                        .find(|b| b.ticker == "USDC".to_string())
                    {
                        balance.locked_balance += required_amount;
                    }
                    true
                } else {
                    false
                }
            }
            OrderSide::Sell => {
                let sol_balance = user
                    .balances
                    .iter()
                    .find(|b| b.ticker == "SOL".to_string())
                    .map(|b| b.balance)
                    .unwrap_or(dec!(0));
                if sol_balance >= payload.quantity {
                    if let Some(balance) = user
                        .balances
                        .iter_mut()
                        .find(|b| b.ticker == "SOL".to_string())
                    {
                        balance.locked_balance += payload.quantity;
                    }
                    true
                } else {
                    false
                }
            }
        }
    }
}

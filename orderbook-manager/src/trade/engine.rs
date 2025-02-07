use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::{
    constants::MAX_LEVERAGE,
    models::{
        Balances, CreateOrderPayload, GetQuoteResponse, MessageFromApi, MessageToApi,
        OpenOrdersPayload, Order, OrderCancelledPayload, OrderPlacedPayload, OrderSide, OrderType,
        StatusCode, User,
    },
    utils::redis_manager::RedisManager,
};

use super::Orderbook;

pub struct Engine {
    orderbooks: Arc<Mutex<HashMap<String, Arc<Mutex<Orderbook>>>>>,
    users: Arc<Mutex<Vec<User>>>,
}

impl Engine {
    pub fn new() -> Self {
        let mut orderbooks = HashMap::new();

        let markets = vec![("SOL", "USDC"), ("BTC", "USDC"), ("ETH", "USDC")];

        for (base, quote) in markets {
            let orderbook = Orderbook {
                bids: Vec::new(),
                asks: Vec::new(),
                base_asset: base.to_string(),
                quote_asset: quote.to_string(),
            };
            orderbooks.insert(
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
            orderbooks: Arc::new(Mutex::new(orderbooks)),
            users: Arc::new(Mutex::new(users)),
        }
    }

    pub fn process(&mut self, client_id: String, message: MessageFromApi) {
        match message {
            MessageFromApi::CreateOrder { data } => {
                let result = self.create_order(&data.market, &data);

                match result {
                    Ok((executed_qty, order_id)) => {
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderPlaced {
                            payload: OrderPlacedPayload {
                                order_id,
                                executed_qty,
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
                                message: None,
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                }
            }
            MessageFromApi::CancelOrder { data } => {
                let orderbook = {
                    let orderbooks = self.orderbooks.lock().unwrap();
                    Arc::clone(
                        orderbooks
                            .get(&data.market)
                            .ok_or("Market not found")
                            .unwrap(),
                    )
                };

                let status = {
                    let mut orderbook_guard = orderbook.lock().unwrap();
                    orderbook_guard.find_and_cancel_order(data.user_id.as_str(), &mut self.users)
                };

                match status {
                    StatusCode::OK => {
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderCancelled {
                            payload: OrderCancelledPayload {
                                order_id: data.market,
                                message: Some("Order cancelled".to_string()),
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                    StatusCode::NotFound => {
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderCancelled {
                            payload: OrderCancelledPayload {
                                order_id: data.market,
                                message: Some("Something went wrong".to_string()),
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                }
            }
            MessageFromApi::GetQuote { data } => {
                let orderbooks = self.orderbooks.lock().unwrap();
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
                let orderbooks = self.orderbooks.lock().unwrap();
                let orderbook = orderbooks
                    .get(&data.market)
                    .ok_or("Market not found")
                    .unwrap();

                let orderbook_guard = orderbook.lock().unwrap();
                println!("Market: {}", data.market);
                println!("Number of bids: {}", orderbook_guard.bids.len());
                println!("Number of asks: {}", orderbook_guard.asks.len());
                
                let result = orderbook_guard.get_depth();
                println!("Depth result orders: {}", result.orders.len());

                let redis_manager = RedisManager::instance();
                let message = MessageToApi::Depth { payload: result };

                let _ = redis_manager.send_to_api(&client_id, &message);
            }
            MessageFromApi::GetOpenOrders { data } => {
                let orderbooks = self.orderbooks.lock().unwrap();
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
        }
    }

    fn create_order(
        &mut self,
        market: &str,
        payload: &CreateOrderPayload,
    ) -> Result<(Decimal, String), Box<dyn std::error::Error>> {
        let assets: Vec<&str> = market.split('_').collect();
        if assets.len() != 2 {
            return Err("Invalid market format".into());
        }

        let order_id = Uuid::new_v4().to_string();

        // validation check for spot and margin order
        match payload.order_type {
            OrderType::MarginLong | OrderType::MarginShort => {
                if !self.validate_margin_requirements(&payload) {
                    return Err("Insufficient margin or invalid margin requirements".into());
                }
            }
            OrderType::Spot => {
                if !self.validate_spot_balance(&payload) {
                    return Err("Insufficient balance for spot trade".into());
                }
            }
        }

        // fill order
        let orderbooks = self.orderbooks.lock().unwrap();
        let orderbook = orderbooks
            .get(&payload.market)
            .ok_or("Market not found")
            .unwrap();

        // fill orders
        let remaining_qty = orderbook
            .lock()
            .unwrap()
            .fill_orders(payload, &mut self.users);

        println!("create order orderbook {}", orderbook.lock().unwrap().quote_asset);

        if remaining_qty == Decimal::from(0) {
            return Ok((payload.quantity, order_id.clone()));
        }

        match payload.side {
            OrderSide::Buy => {
                let mut orderbook_guard = orderbook.lock().unwrap();
                orderbook_guard.bids.push(Order {
                    id: order_id.clone(),
                    user_id: payload.user_id.clone(),
                    price: payload.price,
                    quantity: remaining_qty,
                    order_type: payload.order_type.clone(),
                    leverage: payload.leverage,
                });
            }
            OrderSide::Sell => {
                let mut orderbook_guard = orderbook.lock().unwrap();
                orderbook_guard.asks.push(Order {
                    id: order_id.clone(),
                    user_id: payload.user_id.clone(),
                    price: payload.price,
                    quantity: remaining_qty,
                    order_type: payload.order_type.clone(),
                    leverage: payload.leverage,
                });
            }
        }

        Ok((remaining_qty, order_id))
    }

    fn validate_margin_requirements(&self, payload: &CreateOrderPayload) -> bool {
        let mut users = self.users.lock().unwrap();
        let user = users
            .iter_mut()
            .find(|u| u.id == payload.user_id)
            .expect("User not found");

        if !user.margin_enabled {
            return false;
        }

        let leverage = payload.leverage.unwrap_or(dec!(1));
        if leverage > MAX_LEVERAGE {
            return false;
        }

        let usdc_balance = user
            .balances
            .iter()
            .find(|b| b.ticker == "USDC")
            .map(|b| b.balance - b.locked_balance)
            .unwrap_or(dec!(0));

        let required_margin = (payload.price * payload.quantity) / leverage;

        match payload.order_type {
            OrderType::MarginLong => {
                if usdc_balance >= required_margin {
                    if let Some(balance) = user
                        .balances
                        .iter_mut()
                        .find(|b| b.ticker == "USDC".to_string())
                    {
                        balance.locked_balance += required_margin;
                    }
                    return true;
                } else {
                    return false;
                }
            }
            OrderType::MarginShort => {
                let safety_multiplier = dec!(1.1);
                let adjusted_required_margin = required_margin * safety_multiplier;

                if usdc_balance >= adjusted_required_margin {
                    if let Some(balance) = user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        balance.locked_balance += adjusted_required_margin;
                    }
                    return true;
                } else {
                    return false;
                }
            }
            OrderType::Spot => {}
        }
        true
    }

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

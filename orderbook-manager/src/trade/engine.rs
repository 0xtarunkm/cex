use std::sync::{Arc, Mutex};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::{
    constants::MAX_LEVERAGE,
    models::{
        Balances, CreateOrderPayload, MessageFromApi, MessageToApi, Order, OrderCancelledPayload,
        OrderPlacedPayload, OrderSide, OrderType, StatusCode, User,
    },
    utils::redis_manager::RedisManager,
};

use super::Orderbook;

pub struct Engine {
    orderbooks: Arc<Mutex<Vec<Orderbook>>>,
    users: Arc<Mutex<Vec<User>>>,
}

impl Engine {
    pub fn new() -> Self {
        let mut orderbook = Vec::new();
        orderbook.push(Orderbook {
            bids: Vec::new(),
            asks: Vec::new(),
            base_asset: "SOL".to_string(),
            quote_asset: "USDC".to_string(),
        });
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
        ];

        Engine {
            orderbooks: Arc::new(Mutex::new(orderbook)),
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
                let mut orderbook_guard = self.orderbooks.lock().unwrap();
                let orderbook = orderbook_guard
                    .iter_mut()
                    .find(|o| o.quote_asset == data.market)
                    .expect("No orderbook found");

                match orderbook.find_and_cancel_order(data.user_id.as_str(), &mut self.users) {
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
                    StatusCode::NOT_FOUND => {
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
            _ => {}
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
        let mut orderbook_guard = self.orderbooks.lock().unwrap();
        let orderbook = orderbook_guard
            .iter_mut()
            .find(|o| o.quote_asset == market.to_string())
            .expect("No orderbook found");

        // fill orders
        let remaining_qty = orderbook.fill_orders(payload, &mut self.users);

        if remaining_qty == Decimal::from(0) {
            return Ok((payload.quantity, order_id.clone()));
        }

        match payload.side {
            OrderSide::Buy => {
                let mut bids = orderbook.bids.clone();
                bids.push(Order {
                    id: order_id.clone(),
                    user_id: payload.user_id.clone(),
                    price: payload.price,
                    quantity: payload.quantity,
                    order_type: payload.order_type.clone(),
                    leverage: payload.leverage,
                });
            }
            OrderSide::Sell => {
                let mut asks = orderbook.asks.clone();
                asks.push(Order {
                    id: order_id.clone(),
                    user_id: payload.user_id.clone(),
                    price: payload.price,
                    quantity: payload.quantity,
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

use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{self, json};
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    models::{
        Balance, CancelOrderPayload, CreateOrderPayload, MarginOrder, MarginPosition,
        MarginPositionsPayload, MessageFromApi, MessageToApi, OpenOrdersPayload,
        OrderCancelledPayload, OrderPlacedPayload, OrderSide, OrderType, SpotOrder, User,
        UserBalancesPayload,
    },
    services::{     
        pnl_service::PnlMonitor,
        price_service::{PriceInfo, PriceService},
        redis_manager::RedisManager,
    },
};

use super::{MarginOrderbook, SpotOrderbook};

#[allow(dead_code)]
pub struct Engine {
    pub spot_orderbooks: Arc<Mutex<HashMap<String, Arc<Mutex<SpotOrderbook>>>>>,
    pub margin_orderbooks: Arc<Mutex<HashMap<String, Arc<Mutex<MarginOrderbook>>>>>,
    pub users: Arc<Mutex<Vec<User>>>,
    pub price_service: Arc<PriceService>,
}

impl Engine {
    pub fn new() -> Self {
        let mut spot_orderbooks = HashMap::new();
        let mut margin_orderbooks = HashMap::new();

        let markets = vec![("SOL", "USDC"), ("BTC", "USDC"), ("ETH", "USDC")];

        for (base, quote) in &markets {
            let orderbook = SpotOrderbook::new(base.to_string(), quote.to_string());

            spot_orderbooks.insert(
                format!("{}_{}", base, quote),
                Arc::new(Mutex::new(orderbook)),
            );
        }

        for (base, quote) in &markets {
            let orderbook = MarginOrderbook::new(base.to_string(), quote.to_string());

            margin_orderbooks.insert(
                format!("{}_{}", base, quote),
                Arc::new(Mutex::new(orderbook)),
            );
        }

        let users = vec![
            {
                let mut user = User::new("1".to_string());
                user.balances = vec![
                    Balance {
                        ticker: "SOL".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balance {
                        ticker: "BTC".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balance {
                        ticker: "ETH".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balance {
                        ticker: "USDC".to_string(),
                        balance: Decimal::from(10_000),
                        locked_balance: Decimal::from(0),
                    },
                ];
                user
            },
            {
                let mut user = User::new("2".to_string());
                user.balances = vec![
                    Balance {
                        ticker: "SOL".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balance {
                        ticker: "BTC".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balance {
                        ticker: "ETH".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balance {
                        ticker: "USDC".to_string(),
                        balance: Decimal::from(10_000),
                        locked_balance: Decimal::from(0),
                    },
                ];
                user
            },
            {
                let mut user = User::new("3".to_string());
                user.balances = vec![
                    Balance {
                        ticker: "SOL".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balance {
                        ticker: "BTC".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balance {
                        ticker: "ETH".to_string(),
                        balance: Decimal::from(100),
                        locked_balance: Decimal::from(0),
                    },
                    Balance {
                        ticker: "USDC".to_string(),
                        balance: Decimal::from(10_000),
                        locked_balance: Decimal::from(0),
                    },
                ];
                user
            },
        ];

        let spot_orderbooks = Arc::new(Mutex::new(spot_orderbooks));
        let margin_orderbooks = Arc::new(Mutex::new(margin_orderbooks));
        let users = Arc::new(Mutex::new(users));

        let price_service = Arc::new(PriceService::new(
            spot_orderbooks.clone(),
            margin_orderbooks.clone(),
        ));

        let price_service_clone = Arc::clone(&price_service);
        tokio::spawn(async move {
            price_service_clone.start_price_updates().await;
        });

        let pnl_monitor = PnlMonitor::new(Arc::clone(&users), Arc::clone(&price_service));

        tokio::spawn(async move {
            pnl_monitor.start_monitoring().await;
        });

        Engine {
            spot_orderbooks,
            margin_orderbooks,
            users,
            price_service,
        }
    }

    pub async fn process(&mut self, client_id: String, message: MessageFromApi) {
        info!(?message, "Processing message from client {}", client_id);
        match message {
            MessageFromApi::CreateOrder { data } => {
                info!(?data, "Creating order");
                let result = self.create_order(&data).await;

                match result {
                    Ok((remaining_qty, filled_qty, order_id)) => {
                        info!(
                            order_id,
                            ?remaining_qty,
                            ?filled_qty,
                            "Order created successfully"
                        );
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderPlaced {
                            payload: OrderPlacedPayload {
                                order_id,
                                remaining_qty,
                                filled_qty,
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                        let trade_info = json!({
                            "price": data.price,
                            "quantity": filled_qty,
                            "side": data.side,
                            "timestamp": Utc::now().timestamp()
                        });
                        let _ = redis_manager
                            .publish_message(&format!("trade@{}", data.market), &trade_info);
                    }
                    Err(e) => {
                        error!("Failed to create order: {}", e);
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
                info!(?data, "Cancelling order");
                let result = self.cancel_order(&data).await;

                match result {
                    Ok(()) => {
                        info!(order_id = ?data.order_id, "Order cancelled successfully");
                        let redis_manager = RedisManager::instance();
                        let message = MessageToApi::OrderCancelled {
                            payload: OrderCancelledPayload {
                                message: Some(String::from("ORDER CANCELLED")),
                            },
                        };

                        let _ = redis_manager.send_to_api(&client_id, &message);
                    }
                    Err(e) => {
                        error!("Failed to cancel order: {}", e);
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
                let result = match data.order_type {
                    OrderType::MarginLong | OrderType::MarginShort => {
                        let orderbooks = self.margin_orderbooks.lock().await;
                        let orderbook = orderbooks
                            .get(&data.market)
                            .ok_or("Market not found")
                            .unwrap();
                        let quote = orderbook
                            .lock()
                            .await
                            .get_quote_detail(data.quantity, data.order_type);
                        quote
                    }
                    OrderType::Spot => {
                        let orderbooks = self.spot_orderbooks.lock().await;
                        let orderbook = orderbooks
                            .get(&data.market)
                            .ok_or("Market not found")
                            .unwrap();
                        let orderbook = orderbook.lock().await;
                        let quote = orderbook.get_quote_detail(data.quantity, data.side);
                        quote
                    }
                };

                let redis_manager = RedisManager::instance();
                let message = MessageToApi::Quote { payload: result };
                let _ = redis_manager.send_to_api(&client_id, &message);
            }
            MessageFromApi::GetDepth { data } => {
                let result = match data.order_type {
                    OrderType::MarginLong | OrderType::MarginShort => {
                        let orderbooks = self.margin_orderbooks.lock().await;
                        let orderbook = orderbooks
                            .get(&data.market)
                            .ok_or("Market not found")
                            .unwrap();
                        let depth = orderbook.lock().await.get_depth();
                        depth
                    }
                    OrderType::Spot => {
                        let orderbooks = self.spot_orderbooks.lock().await;
                        let orderbook = orderbooks
                            .get(&data.market)
                            .ok_or("Market not found")
                            .unwrap();
                        let depth = orderbook.lock().await.get_depth();
                        depth
                    }
                };

                let redis_manager = RedisManager::instance();
                let message = MessageToApi::Depth { payload: result };
                let _ = redis_manager.send_to_api(&client_id, &message);
            }
            MessageFromApi::GetOpenOrders { data } => {
                let orderbooks = self.spot_orderbooks.lock().await;
                let orderbook = orderbooks
                    .get(&data.market)
                    .ok_or("Market not found")
                    .unwrap();

                let mut open_orders = Vec::new();

                for bid in orderbook.lock().await.bids.iter() {
                    if bid.user_id == data.user_id {
                        open_orders.push(bid.clone());
                    }
                }

                for ask in orderbook.lock().await.asks.iter() {
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
                let mut users = self.users.lock().await;
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
            MessageFromApi::GetMarginPositions { data } => {
                let mut users = self.users.lock().await;
                info!(?data, "Getting margin positions for user {}", data.user_id);

                let user = users
                    .iter_mut()
                    .find(|u| u.id == data.user_id)
                    .expect("User not found");

                let redis_manager = RedisManager::instance();
                let message = MessageToApi::GetMarginPositions {
                    payload: MarginPositionsPayload {
                        positions: user.margin_positions.clone(),
                    },
                };

                let _ = redis_manager.send_to_api(&client_id, &message);
            }
            MessageFromApi::GetTicker { market, order_type } => {
                let price = match order_type {
                    OrderType::MarginLong | OrderType::MarginShort => {
                        let orderbooks = self.margin_orderbooks.lock().await;
                        let orderbook = orderbooks.get(&market).ok_or("Market not found").unwrap();
                        let orderbook = orderbook.lock().await;
                        let price = orderbook.get_price_info().await;
                        price
                    }
                    OrderType::Spot => {
                        let orderbooks = self.spot_orderbooks.lock().await;
                        let orderbook = orderbooks.get(&market).ok_or("Market not found").unwrap();
                        let orderbook = orderbook.lock().await;
                        let price = orderbook.get_price_info().await;
                        price
                    }
                };
                let redis_manager = RedisManager::instance();
                let message = MessageToApi::TickerPrice {
                    market,
                    price: price.map(|p| PriceInfo {
                        last_trade_price: p.last_trade_price,
                        mark_price: p.mark_price,
                        index_price: p.index_price,
                        timestamp: p.timestamp,
                    }),
                };
                let _ = redis_manager.send_to_api(&client_id, &message);
            }
        }
    }

    pub async fn create_order(
        &mut self,
        payload: &CreateOrderPayload,
    ) -> Result<(Decimal, Decimal, String), Box<dyn std::error::Error>> {
        let order_id = Uuid::new_v4().to_string();

        match payload.order_type {
            OrderType::MarginLong | OrderType::MarginShort => {
                if !self.validate_margin_requirements(&payload).await {
                    error!("Insufficient margin or invalid margin requirements");
                    return Err("Insufficient margin or invalid margin requirements".into());
                }

                let orderbooks = self.margin_orderbooks.lock().await;
                let orderbook = orderbooks
                    .get(&payload.market)
                    .ok_or("Market not found")
                    .unwrap();

                let remaining_qty = orderbook
                    .lock()
                    .await
                    .fill_orders(
                        payload,
                        &mut self.users,
                        &payload.market.split('_').nth(0).unwrap().to_string(),
                    )
                    .await;

                if remaining_qty == Decimal::from(0) {
                    return Ok((Decimal::ZERO, payload.quantity, order_id.clone()));
                }

                let mut users = self.users.lock().await;
                if let Some(user) = users.iter_mut().find(|u| u.id == payload.user_id) {
                    let base_asset = payload.market.split('_').nth(0).unwrap().to_string();
                    let new_position = MarginPosition {
                        asset: base_asset,
                        quantity: remaining_qty,
                        avg_price: payload.price,
                        position_type: payload.order_type,
                        unrealized_pnl: None,
                        liquidation_price: None,
                    };
                    user.margin_positions.push(new_position);
                }

                let mut orderbook_guard = orderbook.lock().await;
                match payload.order_type {
                    OrderType::MarginLong => {
                        orderbook_guard.longs.push(MarginOrder {
                            id: order_id.clone(),
                            user_id: payload.user_id.clone(),
                            price: payload.price,
                            quantity: remaining_qty,
                            leverage: payload.leverage.unwrap_or(dec!(1)),
                            order_type: OrderType::MarginLong,
                            timestamp: Utc::now().timestamp(),
                        });

                        orderbook_guard.longs.sort_by(|a, b| {
                            b.price
                                .cmp(&a.price)
                                .then_with(|| a.timestamp.cmp(&b.timestamp))
                        });
                    }
                    OrderType::MarginShort => {
                        orderbook_guard.shorts.push(MarginOrder {
                            id: order_id.clone(),
                            user_id: payload.user_id.clone(),
                            price: payload.price,
                            quantity: remaining_qty,
                            leverage: payload.leverage.unwrap_or(dec!(1)),
                            order_type: OrderType::MarginShort,
                            timestamp: Utc::now().timestamp(),
                        });

                        orderbook_guard.shorts.sort_by(|a, b| {
                            a.price
                                .cmp(&b.price)
                                .then_with(|| a.timestamp.cmp(&b.timestamp))
                        });
                    }
                    _ => unreachable!(),
                }

                let filled_qty = payload.quantity.checked_sub(remaining_qty).unwrap();
                info!(
                    remaining_qty = ?remaining_qty,
                    filled_qty = ?filled_qty,
                    order_id = ?order_id,
                    "Order created successfully"
                );

                // Publish updates
                let redis_manager = RedisManager::instance();
                let depth = orderbook_guard.get_depth();
                let price_info = orderbook_guard.get_price_info().await;

                let _ = redis_manager.publish_message(
                    &format!("depth@{}", payload.market),
                    &serde_json::to_value(depth).unwrap(),
                );

                if let Some(price) = price_info {
                    let _ = redis_manager.publish_message(
                        &format!("ticker@{}", payload.market),
                        &serde_json::to_value(price).unwrap(),
                    );
                }

                Ok((remaining_qty, filled_qty, order_id))
            }
            OrderType::Spot => {
                if !self.validate_spot_balance(&payload).await {
                    error!("Insufficient balance for spot trade");
                    return Err("Insufficient balance for spot trade".into());
                }

                let orderbooks = self.spot_orderbooks.lock().await;
                let orderbook = orderbooks
                    .get(&payload.market)
                    .ok_or("Market not found")
                    .unwrap();

                let remaining_qty = orderbook
                    .lock()
                    .await
                    .fill_orders(
                        payload,
                        &mut self.users,
                        &payload.market.split('_').nth(0).unwrap().to_string(),
                        &payload.market.split('_').nth(1).unwrap().to_string(),
                    )
                    .await;

                if remaining_qty == Decimal::from(0) {
                    return Ok((Decimal::ZERO, payload.quantity, order_id.clone()));
                }

                match payload.side {
                    OrderSide::Buy => {
                        let mut orderbook_guard = orderbook.lock().await;
                        orderbook_guard.bids.push(SpotOrder {
                            id: order_id.clone(),
                            user_id: payload.user_id.clone(),
                            price: payload.price,
                            quantity: remaining_qty,
                            side: payload.side.clone(),
                            timestamp: Utc::now().timestamp(),
                        });

                        orderbook_guard.bids.sort_by(|a, b| {
                            b.price
                                .cmp(&a.price)
                                .then_with(|| a.timestamp.cmp(&b.timestamp))
                        });

                        // Publish updates
                        let redis_manager = RedisManager::instance();
                        let depth = orderbook_guard.get_depth();
                        let price_info = orderbook_guard.get_price_info().await;

                        let _ = redis_manager.publish_message(
                            &format!("depth@{}", payload.market),
                            &serde_json::to_value(depth).unwrap(),
                        );

                        if let Some(price) = price_info {
                            let _ = redis_manager.publish_message(
                                &format!("ticker@{}", payload.market),
                                &serde_json::to_value(price).unwrap(),
                            );
                        }
                    }
                    OrderSide::Sell => {
                        let mut orderbook_guard = orderbook.lock().await;
                        orderbook_guard.asks.push(SpotOrder {
                            id: order_id.clone(),
                            user_id: payload.user_id.clone(),
                            price: payload.price,
                            quantity: remaining_qty,
                            side: payload.side.clone(),
                            timestamp: Utc::now().timestamp(),
                        });

                        orderbook_guard.asks.sort_by(|a, b| {
                            a.price
                                .cmp(&b.price)
                                .then_with(|| a.timestamp.cmp(&b.timestamp))
                        });

                        // Publish updates
                        let redis_manager = RedisManager::instance();
                        let depth = orderbook_guard.get_depth();
                        let price_info = orderbook_guard.get_price_info().await;

                        let _ = redis_manager.publish_message(
                            &format!("depth@{}", payload.market),
                            &serde_json::to_value(depth).unwrap(),
                        );

                        if let Some(price) = price_info {
                            let _ = redis_manager.publish_message(
                                &format!("ticker@{}", payload.market),
                                &serde_json::to_value(price).unwrap(),
                            );
                        }
                    }
                }

                let filled_qty = payload.quantity.checked_sub(remaining_qty).unwrap();

                info!(
                    remaining_qty = ?remaining_qty,
                    filled_qty = ?filled_qty,
                    order_id = ?order_id,
                    "Order created successfully"
                );
                Ok((remaining_qty, filled_qty, order_id))
            }
        }
    }

    pub async fn cancel_order(
        &mut self,
        payload: &CancelOrderPayload,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let orderbooks = self.spot_orderbooks.lock().await;
        let orderbook = orderbooks.get(&payload.market).ok_or("Market not found")?;

        let mut orderbook_guard = orderbook.lock().await;

        if let Some(bid_index) = orderbook_guard
            .bids
            .iter()
            .position(|order| order.id == payload.order_id)
        {
            let order = &orderbook_guard.bids[bid_index];

            let mut users = self.users.lock().await;
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

            let mut users = self.users.lock().await;
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

    async fn validate_margin_requirements(&self, payload: &CreateOrderPayload) -> bool {
        let mut users = self.users.lock().await;
        let user = users
            .iter_mut()
            .find(|u| u.id == payload.user_id)
            .expect("User not found");

        if !user.margin_enabled {
            warn!(user_id = ?user.id, "Margin trading not enabled for user");
            return false;
        }

        let leverage = payload.leverage.unwrap_or(dec!(1));
        if leverage > user.max_leverage {
            warn!(
                user_id = ?user.id,
                requested = ?leverage,
                max = ?user.max_leverage,
                "Requested leverage exceeds maximum"
            );
            return false;
        }

        let usdc_balance = user
            .balances
            .iter()
            .find(|b| b.ticker == "USDC")
            .map(|b| b.balance - b.locked_balance)
            .unwrap_or(dec!(0));

        let position_value = payload.price * payload.quantity;
        let required_margin = position_value / leverage;

        let total_margin_used = user.margin_used + required_margin;
        let max_margin_allowed = usdc_balance * user.max_leverage;

        if total_margin_used > max_margin_allowed {
            return false;
        }

        match payload.order_type {
            OrderType::MarginLong => {
                if let Some(existing_short) = user.margin_positions.iter().find(|p| {
                    p.asset == payload.market.split('_').next().unwrap()
                        && p.position_type == OrderType::MarginShort
                }) {
                    if existing_short.quantity >= payload.quantity {
                        return true;
                    }
                }

                if usdc_balance >= required_margin {
                    if let Some(balance) = user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        balance.locked_balance += required_margin;
                        return true;
                    }
                }
            }
            OrderType::MarginShort => {
                if let Some(existing_long) = user.margin_positions.iter().find(|p| {
                    p.asset == payload.market.split('_').next().unwrap()
                        && p.position_type == OrderType::MarginLong
                }) {
                    if existing_long.quantity >= payload.quantity {
                        return true;
                    }
                }

                let safety_multiplier = dec!(1.1);
                let adjusted_required_margin = required_margin * safety_multiplier;

                if usdc_balance >= adjusted_required_margin {
                    if let Some(balance) = user.balances.iter_mut().find(|b| b.ticker == "USDC") {
                        balance.locked_balance += adjusted_required_margin;
                        return true;
                    }
                }
            }
            OrderType::Spot => return true,
        }

        false
    }

    async fn validate_spot_balance(&self, payload: &CreateOrderPayload) -> bool {
        let mut users = self.users.lock().await;
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

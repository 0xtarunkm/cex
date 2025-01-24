use crate::messages::MessageFromApi;
use crate::models::{Fill, Order, Orderbook, Side, UserBalance};
use rand::Rng;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const BASE_CURRENCY: &str = "USDC";

pub struct Engine {
    orderbooks: Arc<Mutex<Vec<Orderbook>>>,
    balances: Arc<Mutex<HashMap<String, HashMap<String, UserBalance>>>>,
}

impl Engine {
    pub fn new() -> Self {
        let orderbooks = Arc::new(Mutex::new(vec![Orderbook::new(
            "TATA".to_string(),
            vec![],
            vec![],
            0,
            Decimal::ZERO,
        )]));
        let balances = Arc::new(Mutex::new(HashMap::new()));
        let engine = Engine {
            orderbooks,
            balances,
        };

        tokio::spawn(async move {});
        engine
    }

    pub async fn process(&self, message: MessageFromApi, client_id: String) {
        match message {
            MessageFromApi::CreateOrder {
                market,
                price,
                quantity,
                side,
                user_id,
            } => {
                if let Err(e) = self
                    .create_order(&market, price, quantity, side, &user_id)
                    .await
                {
                    eprintln!("Error creating order: {}", e);
                }
            }
            MessageFromApi::CancelOrder {
                order_id,
                market,
                user_id,
            } => {
                if let Err(e) = self.cancel_order(&order_id, &market, &user_id).await {
                    eprintln!("Error canceling order: {}", e);
                }
            }
            _ => {}
        }
    }

    pub async fn create_order(
        &self,
        market: &str,
        price: Decimal,
        quantity: Decimal,
        side: Side,
        user_id: &str,
    ) -> Result<(), String> {
        let mut orderbooks = self.orderbooks.lock().await;
        let orderbook = orderbooks
            .iter_mut()
            .find(|o| o.ticker() == market)
            .ok_or("Orderbook not found")?;

        let base_asset = market.split('_').next().ok_or("Invalid market")?;
        let quote_asset = market.split('_').nth(1).ok_or("Invalid market")?;

        self.check_and_lock_funds(base_asset, quote_asset, side, user_id, price, quantity)
            .await?;

        // Generate a random order ID
        let order_id = format!("order_{}", rand::thread_rng().gen::<u64>());

        let order = Order {
            price,
            quantity,
            order_id,
            filled: Decimal::ZERO,
            side,
            user_id: user_id.to_string(),
        };

        let (fills, executed_qty) = orderbook.add_order(order)?;
        self.update_balance(user_id, base_asset, quote_asset, side, &fills, executed_qty)
            .await;

        Ok(())
    }

    pub async fn cancel_order(
        &self,
        order_id: &str,
        market: &str,
        user_id: &str,
    ) -> Result<(), String> {
        let mut orderbooks = self.orderbooks.lock().await;
        let orderbook = orderbooks
            .iter_mut()
            .find(|o| o.ticker() == market)
            .ok_or("Orderbook not found")?;

        let order = orderbook
            .asks
            .iter()
            .flat_map(|(_, orders)| orders.iter())
            .find(|order| order.order_id == order_id)
            .or_else(|| {
                orderbook
                    .bids
                    .iter()
                    .flat_map(|(_, orders)| orders.iter())
                    .find(|order| order.order_id == order_id)
            })
            .cloned()
            .ok_or("Order not found")?;

        let price = if order.side == Side::Buy {
            orderbook
                .cancel_bid(order_id)
                .ok_or("Failed to cancel bid")?
        } else {
            orderbook
                .cancel_ask(order_id)
                .ok_or("Failed to cancel ask")?
        };

        let left_quantity = order.quantity - order.filled;
        let mut balances = self.balances.lock().await;
        let user_balance = balances.get_mut(user_id).ok_or("User balance not found")?;

        if order.side == Side::Buy {
            user_balance.get_mut(BASE_CURRENCY).unwrap().available += left_quantity * order.price;
            user_balance.get_mut(BASE_CURRENCY).unwrap().locked -= left_quantity * order.price;
        } else {
            user_balance
                .get_mut(market.split('_').next().unwrap())
                .unwrap()
                .available += left_quantity;
            user_balance
                .get_mut(market.split('_').next().unwrap())
                .unwrap()
                .locked -= left_quantity;
        }

        Ok(())
    }

    async fn check_and_lock_funds(
        &self,
        base_asset: &str,
        quote_asset: &str,
        side: Side,
        user_id: &str,
        price: Decimal,
        quantity: Decimal,
    ) -> Result<(), String> {
        let mut balances = self.balances.lock().await;
        let user_balance = balances.get_mut(user_id).ok_or("User balance not found")?;

        match side {
            Side::Buy => {
                let required = price * quantity;
                if user_balance
                    .get(quote_asset)
                    .map(|b| b.available)
                    .unwrap_or(Decimal::ZERO)
                    < required
                {
                    return Err("Insufficient funds".to_string());
                }
                user_balance.get_mut(quote_asset).unwrap().available -= required;
                user_balance.get_mut(quote_asset).unwrap().locked += required;
            }
            Side::Sell => {
                if user_balance
                    .get(base_asset)
                    .map(|b| b.available)
                    .unwrap_or(Decimal::ZERO)
                    < quantity
                {
                    return Err("Insufficient funds".to_string());
                }
                user_balance.get_mut(base_asset).unwrap().available -= quantity;
                user_balance.get_mut(base_asset).unwrap().locked += quantity;
            }
        }

        Ok(())
    }

    async fn update_balance(
        &self,
        user_id: &str,
        base_asset: &str,
        quote_asset: &str,
        side: Side,
        fills: &[Fill],
        executed_qty: Decimal,
    ) {
        let mut balances = self.balances.lock().await;
        let user_balance = balances.get_mut(user_id).unwrap();

        for fill in fills {
            match side {
                Side::Buy => {
                    user_balance.get_mut(quote_asset).unwrap().available += fill.qty * fill.price;
                    user_balance.get_mut(quote_asset).unwrap().locked -= fill.qty * fill.price;
                    user_balance.get_mut(base_asset).unwrap().available += fill.qty;
                }
                Side::Sell => {
                    user_balance.get_mut(quote_asset).unwrap().locked -= fill.qty * fill.price;
                    user_balance.get_mut(quote_asset).unwrap().available += fill.qty * fill.price;
                    user_balance.get_mut(base_asset).unwrap().locked -= fill.qty;
                }
            }
        }
    }

    fn set_base_balances(&self) {
        let mut balances = self.balances.blocking_lock();
        balances.insert(
            "1".to_string(),
            HashMap::from([
                (
                    BASE_CURRENCY.to_string(),
                    UserBalance {
                        available: Decimal::from(10_000_000),
                        locked: Decimal::ZERO,
                    },
                ),
                (
                    "TATA".to_string(),
                    UserBalance {
                        available: Decimal::from(10_000_000),
                        locked: Decimal::ZERO,
                    },
                ),
            ]),
        );
        balances.insert(
            "2".to_string(),
            HashMap::from([
                (
                    BASE_CURRENCY.to_string(),
                    UserBalance {
                        available: Decimal::from(10_000_000),
                        locked: Decimal::ZERO,
                    },
                ),
                (
                    "TATA".to_string(),
                    UserBalance {
                        available: Decimal::from(10_000_000),
                        locked: Decimal::ZERO,
                    },
                ),
            ]),
        );
        balances.insert(
            "5".to_string(),
            HashMap::from([
                (
                    BASE_CURRENCY.to_string(),
                    UserBalance {
                        available: Decimal::from(10_000_000),
                        locked: Decimal::ZERO,
                    },
                ),
                (
                    "TATA".to_string(),
                    UserBalance {
                        available: Decimal::from(10_000_000),
                        locked: Decimal::ZERO,
                    },
                ),
            ]),
        );
    }
}

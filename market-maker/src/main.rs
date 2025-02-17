use std::time::Duration;

use anyhow::Result;
use constants::{BASE_URL, MARKET, TOTAL_ASKS, TOTAL_BIDS, USER_ID};
use models::{CreateOrderPayload, MessageFromEngine, OrderSide, OrderType, SpotOrder};
use rand::Rng;
use reqwest::Client;
use rust_decimal::{prelude::FromPrimitive, Decimal};

mod constants;
mod models;

async fn order_loop() -> Result<()> {
    let client = Client::new();
    loop {
        let mut rng = rand::rng();
        let price = Decimal::from_f64(10.0 + rng.random::<f64>() * 10.0).unwrap();

        let response: MessageFromEngine = client
            .get(format!(
                "{}/api/v1/order/open?user_id={}&market={}",
                BASE_URL, USER_ID, MARKET
            ))
            .send()
            .await?
            .json()
            .await?;

        let open_orders = match response {
            MessageFromEngine::OpenOrders { payload } => payload.open_orders,
            _ => Vec::new(),
        };

        let total_bids = open_orders
            .iter()
            .filter(|o| o.side == OrderSide::Buy)
            .count() as i32;
        let total_asks = open_orders
            .iter()
            .filter(|o| o.side == OrderSide::Sell)
            .count() as i32;

        // cancel existing orders
        let cancelled_bids = cancel_bids_more_than(&client, &open_orders, price).await?;
        let cancelled_asks = cancel_asks_less_than(&client, &open_orders, price).await?;

        let mut bids_to_add = TOTAL_BIDS - total_bids - cancelled_bids;
        let mut asks_to_add = TOTAL_ASKS - total_asks - cancelled_asks;

        while bids_to_add > 0 || asks_to_add > 0 {
            if bids_to_add > 0 {
                let bid_price = price - Decimal::from_f64(rng.random::<f64>() * 1.0).unwrap();
                client
                    .post(format!("{}/api/v1/order/create", BASE_URL))
                    .json(&CreateOrderPayload {
                        market: MARKET.to_string(),
                        price: bid_price,
                        quantity: Decimal::ONE,
                        side: OrderSide::Buy,
                        user_id: USER_ID.to_string(),
                        leverage: None,
                        order_type: OrderType::Spot,
                    })
                    .send()
                    .await?;
                bids_to_add -= 1;
            }

            if asks_to_add > 0 {
                let ask_price = price + Decimal::from_f64(rng.random::<f64>() * 1.0).unwrap();
                client
                    .post(format!("{}/api/v1/order/create", BASE_URL))
                    .json(&CreateOrderPayload {
                        market: MARKET.to_string(),
                        price: ask_price,
                        quantity: Decimal::ONE,
                        side: OrderSide::Sell,
                        user_id: USER_ID.to_string(),
                        leverage: None,
                        order_type: OrderType::Spot,
                    })
                    .send()
                    .await?;
                asks_to_add -= 1;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn cancel_bids_more_than(
    client: &Client,
    open_orders: &[SpotOrder],
    price: Decimal,
) -> Result<i32> {
    let mut cancelled = 0;
    let mut rng = rand::rng();

    for order in open_orders {
        if order.side == OrderSide::Buy {
            let order_price: Decimal = order.price;
            if order_price < price || rng.random::<f64>() < 0.5 {
                client
                    .delete(format!("{}/api/v1/order/cancel", BASE_URL))
                    .json(&serde_json::json!({
                        "order_id": order.id,
                        "user_id": "3",
                        "market": MARKET
                    }))
                    .send()
                    .await?;
                cancelled += 1;
            }
        }
    }
    Ok(cancelled)
}

async fn cancel_asks_less_than(
    client: &Client,
    open_orders: &[SpotOrder],
    price: Decimal,
) -> Result<i32> {
    let mut cancelled = 0;
    let mut rng = rand::rng();

    for order in open_orders {
        if order.side == OrderSide::Sell {
            let order_price: Decimal = order.price;
            if order_price < price || rng.random::<f64>() < 0.5 {
                client
                    .delete(format!("{}/api/v1/order/cancel", BASE_URL))
                    .json(&serde_json::json!({
                        "order_id": order.id,
                        "user_id": "3",
                        "market": MARKET
                    }))
                    .send()
                    .await?;
                cancelled += 1;
            }
        }
    }
    Ok(cancelled)
}

#[tokio::main]
async fn main() -> Result<()> {
    order_loop().await
}

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Order {
    pub id: u64,
    pub user_id: u64,
    pub price: u64,
    pub quantity: u64,
    pub side: OrderSide,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug)]
pub struct Orderbook {
    pub buy_orders: BTreeMap<u64, Vec<Order>>,
    pub sell_orders: BTreeMap<u64, Vec<Order>>,
}

impl Orderbook {
    pub fn new() -> Self {
        Orderbook {
            buy_orders: BTreeMap::new(),
            sell_orders: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        match order.side {
            OrderSide::Buy => {
                self.buy_orders
                    .entry(order.price)
                    .or_insert_with(Vec::new)
                    .push(order);
            }
            OrderSide::Sell => {
                self.sell_orders
                    .entry(order.price)
                    .or_insert_with(Vec::new)
                    .push(order);
            }
        }
    }

    pub fn remove_order(&mut self, order_id: u64, price: u64, side: OrderSide) -> Option<Order> {
        match side {
            OrderSide::Buy => {
                if let Some(orders) = self.buy_orders.get_mut(&price) {
                    if let Some(pos) = orders.iter().position(|order| order.id == order_id) {
                        return Some(orders.remove(pos));
                    }
                }
            }
            OrderSide::Sell => {
                if let Some(orders) = self.sell_orders.get_mut(&price) {
                    if let Some(pos) = orders.iter().position(|order| order.id == order_id) {
                        return Some(orders.remove(pos));
                    }
                }
            }
        }
        None
    }
}

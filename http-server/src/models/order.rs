use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateOrder {
    market: String,
    price: f64,
    quantity: f64,
    side: Side,
    user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteOrder {
    order_id: String,
    market: String,
}

#[derive(Debug, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

use crate::models::Side;
use rust_decimal::Decimal;

#[derive(Debug)]
pub enum MessageFromApi {
    CreateOrder {
        market: String,
        price: Decimal,
        quantity: Decimal,
        side: Side,
        user_id: String,
    },
    CancelOrder {
        order_id: String,
        market: String,
        user_id: String,
    },
}

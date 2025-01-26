use crate::trade::Fill;

#[derive(Debug)]
pub struct MessageToApi {
    executed_qty: f64,
    order_id: String,
}

use super::{Balance, MarginPosition};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub balances: Vec<Balance>,
    pub margin_positions: Vec<MarginPosition>,
    pub margin_enabled: bool,
    pub margin_used: Decimal,
    pub max_leverage: Decimal,
    pub realized_pnl: Decimal,
}

impl User {
    pub fn _new(id: String) -> Self {
        User {
            id,
            balances: Vec::new(),
            margin_positions: Vec::new(),
            margin_enabled: true,
            margin_used: Decimal::ZERO,
            max_leverage: dec!(10),
            realized_pnl: Decimal::ZERO,
        }
    }
}

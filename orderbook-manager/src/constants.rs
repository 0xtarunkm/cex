use once_cell::sync::Lazy;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::models::LeverageTier;

pub const TICKER: &str = "SOL";
pub const INITIAL_MARGIN_REQUIREMENT: f64 = 0.5;
pub const MAINTENANCE_MARGIN_REQUIREMENT: f64 = 0.25;
pub const MAX_LEVERAGE: i64 = 10;
pub const LIQUIDATION_PENALTY: f64 = 0.05;
pub static LEVERAGE_TIERS: Lazy<HashMap<u32, LeverageTier>> = Lazy::new(|| {
    HashMap::from([
        (
            2,
            LeverageTier {
                initial_margin: Decimal::new(5, 1),
                maintenance_margin: Decimal::new(25, 2),
            },
        ),
        (
            5,
            LeverageTier {
                initial_margin: Decimal::new(2, 1),
                maintenance_margin: Decimal::new(1, 1),
            },
        ),
        (
            10,
            LeverageTier {
                initial_margin: Decimal::new(1, 1),
                maintenance_margin: Decimal::new(5, 2),
            },
        ),
    ])
});

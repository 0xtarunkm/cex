use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub const LIQUIDATION_THRESHOLD: Decimal = dec!(0.15);
pub const MAX_LEVERAGE: Decimal = dec!(10);

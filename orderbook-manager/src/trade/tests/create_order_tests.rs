use rust_decimal_macros::dec;
use crate::models::{CreateOrderPayload, OrderSide, OrderType};
use crate::trade::Engine;

#[test]
fn test_create_spot_buy_order() {
    let mut engine = Engine::new();
    
    let payload = CreateOrderPayload {
        user_id: "1".to_string(),
        market: "SOL_USDC".to_string(),
        price: dec!(20),
        quantity: dec!(5),
        side: OrderSide::Buy,
        order_type: OrderType::Spot,
        leverage: None,
    };

    let result = engine.create_order(&payload.market, &payload);
    assert!(result.is_ok());
    
    let (executed_qty, order_id) = result.unwrap();
    assert!(!order_id.is_empty());
    assert_eq!(executed_qty, dec!(5));

    // Verify balance is locked
    let users = engine.users.lock().unwrap();
    let user = users.iter().find(|u| u.id == "1").unwrap();
    let usdc_balance = user.balances.iter().find(|b| b.ticker == "USDC").unwrap();
    assert_eq!(usdc_balance.locked_balance, dec!(100)); // 5 * 20 = 100 USDC locked
}

#[test]
fn test_create_margin_long_order() {
    let mut engine = Engine::new();
    
    let payload = CreateOrderPayload {
        user_id: "1".to_string(),
        market: "SOL_USDC".to_string(),
        price: dec!(20),
        quantity: dec!(5),
        side: OrderSide::Buy,
        order_type: OrderType::MarginLong,
        leverage: Some(dec!(5)),
    };

    let result = engine.create_order(&payload.market, &payload);
    assert!(result.is_ok());

    // Verify margin is locked
    let users = engine.users.lock().unwrap();
    let user = users.iter().find(|u| u.id == "1").unwrap();
    assert!(user.margin_used > dec!(0));
}

// Add more tests... 
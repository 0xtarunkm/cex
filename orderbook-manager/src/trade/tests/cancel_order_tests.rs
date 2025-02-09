use rust_decimal_macros::dec;
use crate::models::{CreateOrderPayload, OrderSide, OrderType};
use crate::trade::Engine;

#[test]
fn test_cancel_existing_order() {
    let mut engine = Engine::new();
    
    // First create an order
    let create_payload = CreateOrderPayload {
        user_id: "1".to_string(),
        market: "SOL_USDC".to_string(),
        price: dec!(20),
        quantity: dec!(5),
        side: OrderSide::Buy,
        order_type: OrderType::Spot,
        leverage: None,
    };

    let (_, order_id) = engine.create_order(&create_payload.market, &create_payload).unwrap();
    
    // Then cancel it
    let result = engine.cancel_order(&order_id, "1", "SOL_USDC");
    assert!(result.is_ok());

    // Verify balance is unlocked
    let users = engine.users.lock().unwrap();
    let user = users.iter().find(|u| u.id == "1").unwrap();
    let usdc_balance = user.balances.iter().find(|b| b.ticker == "USDC").unwrap();
    assert_eq!(usdc_balance.locked_balance, dec!(0));
}

// Add more tests... 
use rust_decimal_macros::dec;
use crate::models::{CreateOrderPayload, OrderSide, OrderType};
use crate::trade::{Engine, Orderbook};

#[test]
fn test_get_depth_with_orders() {
    let mut engine = Engine::new();
    
    // Create orders at different price levels
    let orders = vec![
        CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(21),
            quantity: dec!(2),
            side: OrderSide::Sell,
            order_type: OrderType::Spot,
            leverage: None,
        },
        CreateOrderPayload {
            user_id: "2".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(19),
            quantity: dec!(3),
            side: OrderSide::Buy,
            order_type: OrderType::Spot,
            leverage: None,
        },
    ];

    for order in orders {
        let _ = engine.create_order(&order.market, &order);
    }

    let orderbooks = engine.orderbooks.lock().unwrap();
    let depth = Orderbook::get_depth_for_market(&orderbooks, "SOL_USDC").unwrap();
    
    assert!(depth.orders.contains_key(&dec!(21)));
    assert!(depth.orders.contains_key(&dec!(19)));
}

// Add more tests... 
use rust_decimal_macros::dec;
use crate::models::{CreateOrderPayload, OrderSide, OrderType};
use crate::trade::Engine;

// #[test]
// fn test_get_quote() {
//     let mut engine = Engine::new();
    
//     // Create some orders first
//     let orders = vec![
//         CreateOrderPayload {
//             user_id: "1".to_string(),
//             market: "SOL_USDC".to_string(),
//             price: dec!(20),
//             quantity: dec!(2),
//             side: OrderSide::Sell,
//             order_type: OrderType::Spot,
//             leverage: None,
//         },
//         CreateOrderPayload {
//             user_id: "2".to_string(),
//             market: "SOL_USDC".to_string(),
//             price: dec!(21),
//             quantity: dec!(3),
//             side: OrderSide::Sell,
//             order_type: OrderType::Spot,
//             leverage: None,
//         },
//     ];

//     for order in orders {
//         let _ = engine.create_order(&order.market, &order);
//     }

//     let quote = engine.get_quote("SOL_USDC", OrderSide::Buy, dec!(4)).unwrap();
//     assert_eq!(quote.quantity, dec!(4));
//     assert!(quote.avg_price >= dec!(20));
// }

// // Add more tests... 
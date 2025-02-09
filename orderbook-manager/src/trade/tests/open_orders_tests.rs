use rust_decimal_macros::dec;
use crate::models::{CreateOrderPayload, OrderSide, OrderType};
use crate::trade::Engine;

// #[test]
// fn test_get_open_orders() {
//     let mut engine = Engine::new();
    
//     // Create multiple orders
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
//             user_id: "1".to_string(),
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

//     let open_orders = engine.get_open_orders("1", "SOL_USDC").unwrap();
//     assert_eq!(open_orders.len(), 2);
// }

// // Add more tests... 
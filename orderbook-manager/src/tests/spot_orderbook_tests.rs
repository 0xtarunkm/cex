// #[cfg(test)]
// mod tests {
//     use std::sync::{Arc, Mutex};

//     use crate::{
//         models::{Balances, CreateOrderPayload, OrderSide, OrderType, SpotOrder, User},
//         trade::SpotOrderbook,
//     };
//     use rust_decimal_macros::dec;

//     fn setup_test_users() -> Arc<Mutex<Vec<User>>> {
//         let users = vec![
//             User {
//                 id: "1".to_string(),
//                 balances: vec![
//                     Balances {
//                         ticker: "SOL".to_string(),
//                         balance: dec!(100),
//                         locked_balance: dec!(0),
//                     },
//                     Balances {
//                         ticker: "USDC".to_string(),
//                         balance: dec!(10000),
//                         locked_balance: dec!(0),
//                     },
//                 ],
//                 margin_enabled: true,
//                 margin_positions: vec![],
//                 margin_used: dec!(0),
//                 max_leverage: dec!(10),
//                 realized_pnl: dec!(0),
//             },
//             User {
//                 id: "2".to_string(),
//                 balances: vec![
//                     Balances {
//                         ticker: "SOL".to_string(),
//                         balance: dec!(100),
//                         locked_balance: dec!(0),
//                     },
//                     Balances {
//                         ticker: "USDC".to_string(),
//                         balance: dec!(10000),
//                         locked_balance: dec!(0),
//                     },
//                 ],
//                 margin_enabled: true,
//                 margin_positions: vec![],
//                 margin_used: dec!(0),
//                 max_leverage: dec!(10),
//                 realized_pnl: dec!(0),
//             },
//         ];
//         Arc::new(Mutex::new(users))
//     }

//     #[test]
//     fn test_create_order_and_match() {
//         let mut orderbook = SpotOrderbook::new("SOL".to_string(), "USDC".to_string());
//         let mut users = setup_test_users();

//         // User 1 places a sell order
//         let sell_order = CreateOrderPayload {
//             user_id: "1".to_string(),
//             market: "SOL_USDC".to_string(),
//             price: dec!(25),
//             quantity: dec!(10),
//             side: OrderSide::Sell,
//             order_type: OrderType::Spot,
//             leverage: None,
//         };

//         // User 2 places a buy order
//         let buy_order = CreateOrderPayload {
//             user_id: "2".to_string(),
//             market: "SOL_USDC".to_string(),
//             price: dec!(25),
//             quantity: dec!(5),
//             side: OrderSide::Buy,
//             order_type: OrderType::Spot,
//             leverage: None,
//         };

//         let remaining_qty = orderbook.fill_orders(&sell_order, &mut users, "SOL", "USDC");
//         assert_eq!(
//             remaining_qty,
//             dec!(10),
//             "Full sell order should remain unfilled"
//         );
//         assert_eq!(orderbook.asks.len(), 1, "One ask should be in orderbook");

//         let remaining_qty = orderbook.fill_orders(&buy_order, &mut users, "SOL", "USDC");
//         assert_eq!(remaining_qty, dec!(0), "Buy order should be fully filled");
//         assert_eq!(
//             orderbook.asks[0].quantity,
//             dec!(5),
//             "5 SOL should remain in ask"
//         );

//         // Verify balances
//         let users_guard = users.lock().unwrap();
//         let user1 = users_guard.iter().find(|u| u.id == "1").unwrap();
//         let user2 = users_guard.iter().find(|u| u.id == "2").unwrap();

//         // Check User 1 (Seller) balances
//         let user1_sol = user1.balances.iter().find(|b| b.ticker == "SOL").unwrap();
//         let user1_usdc = user1.balances.iter().find(|b| b.ticker == "USDC").unwrap();
//         assert_eq!(user1_sol.balance, dec!(95), "Seller should have 95 SOL");
//         assert_eq!(user1_sol.locked_balance, dec!(5), "5 SOL should be locked");
//         assert_eq!(
//             user1_usdc.balance,
//             dec!(10125),
//             "Seller should have received 125 USDC"
//         );

//         // Check User 2 (Buyer) balances
//         let user2_sol = user2.balances.iter().find(|b| b.ticker == "SOL").unwrap();
//         let user2_usdc = user2.balances.iter().find(|b| b.ticker == "USDC").unwrap();
//         assert_eq!(user2_sol.balance, dec!(105), "Buyer should have 105 SOL");
//         assert_eq!(
//             user2_usdc.balance,
//             dec!(9875),
//             "Buyer should have spent 125 USDC"
//         );
//     }

//     #[test]
//     fn test_order_matching_with_better_price() {
//         let mut orderbook = SpotOrderbook::new("SOL".to_string(), "USDC".to_string());
//         let mut users = setup_test_users();

//         // User 1 places a sell order at 24 USDC
//         let sell_order = CreateOrderPayload {
//             user_id: "1".to_string(),
//             market: "SOL_USDC".to_string(),
//             price: dec!(24),
//             quantity: dec!(10),
//             side: OrderSide::Sell,
//             order_type: OrderType::Spot,
//             leverage: None,
//         };

//         // User 2 places a buy order at 25 USDC
//         let buy_order = CreateOrderPayload {
//             user_id: "2".to_string(),
//             market: "SOL_USDC".to_string(),
//             price: dec!(25),
//             quantity: dec!(5),
//             side: OrderSide::Buy,
//             order_type: OrderType::Spot,
//             leverage: None,
//         };

//         // Place sell order first
//         let remaining_qty = orderbook.fill_orders(&sell_order, &mut users, "SOL", "USDC");
//         assert_eq!(remaining_qty, dec!(10));

//         // Place buy order
//         let remaining_qty = orderbook.fill_orders(&buy_order, &mut users, "SOL", "USDC");
//         assert_eq!(
//             remaining_qty,
//             dec!(0),
//             "Buy order should be fully filled at better price"
//         );

//         // Verify the trade happened at the sell order price (24 USDC)
//         let users_guard = users.lock().unwrap();
//         let user2 = users_guard.iter().find(|u| u.id == "2").unwrap();
//         let user2_usdc = user2.balances.iter().find(|b| b.ticker == "USDC").unwrap();
//         assert_eq!(
//             user2_usdc.balance,
//             dec!(9880),
//             "Buyer should have spent 120 USDC (24 * 5)"
//         );
//     }

//     #[test]
//     fn test_get_depth() {
//         let mut orderbook = SpotOrderbook::new("SOL".to_string(), "USDC".to_string());

//         // Add some orders
//         orderbook.bids.push(SpotOrder {
//             id: "1".to_string(),
//             user_id: "1".to_string(),
//             price: dec!(24),
//             quantity: dec!(5),
//             side: OrderSide::Buy,
//             timestamp: 1,
//         });
//         orderbook.bids.push(SpotOrder {
//             id: "2".to_string(),
//             user_id: "1".to_string(),
//             price: dec!(24),
//             quantity: dec!(3),
//             side: OrderSide::Buy,
//             timestamp: 2,
//         });
//         orderbook.asks.push(SpotOrder {
//             id: "3".to_string(),
//             user_id: "2".to_string(),
//             price: dec!(25),
//             quantity: dec!(4),
//             side: OrderSide::Sell,
//             timestamp: 3,
//         });

//         let depth = orderbook.get_depth();

//         assert_eq!(depth.orders.len(), 2, "Should have two price levels");
//         assert_eq!(
//             depth.orders.get(&dec!(24)).unwrap().quantity,
//             dec!(8),
//             "Should aggregate quantities at same price"
//         );
//         assert_eq!(depth.orders.get(&dec!(25)).unwrap().quantity, dec!(4));
//     }
// }

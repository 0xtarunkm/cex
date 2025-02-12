#[cfg(test)]
mod margin_orderbook_tests {
    use crate::{
        models::{CreateOrderPayload, MessageFromApi, OrderSide, OrderType},
        trade::Engine,
        services::pnl_service::PnlMonitor,
    };
    use rust_decimal_macros::dec;
    use std::time::Duration;
    use tokio::time::sleep;
    use tracing::info;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_margin_long_order() {
        let mut engine = Engine::new();

        let order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(2),
            side: OrderSide::Buy,
            order_type: OrderType::MarginLong,
            leverage: Some(dec!(5)),
        };

        let message = MessageFromApi::CreateOrder { data: order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.margin_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.longs.len(), 1);
        assert_eq!(orderbook.longs[0].price, dec!(20));
        assert_eq!(orderbook.longs[0].quantity, dec!(2));
        assert_eq!(orderbook.longs[0].leverage, dec!(5));
    }

    #[tokio::test]
    async fn test_create_margin_short_order() {
        let mut engine = Engine::new();

        let order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(22),
            quantity: dec!(1),
            side: OrderSide::Sell,
            order_type: OrderType::MarginShort,
            leverage: Some(dec!(3)),
        };

        let message = MessageFromApi::CreateOrder { data: order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.margin_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.shorts.len(), 1);
        assert_eq!(orderbook.shorts[0].price, dec!(22));
        assert_eq!(orderbook.shorts[0].quantity, dec!(1));
        assert_eq!(orderbook.shorts[0].leverage, dec!(3));
    }

    #[tokio::test]
    async fn test_margin_order_matching() {
        let mut engine = Engine::new();

        let short_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(2),
            side: OrderSide::Sell,
            order_type: OrderType::MarginShort,
            leverage: Some(dec!(5)),
        };

        let message = MessageFromApi::CreateOrder { data: short_order };
        engine.process("test_client".to_string(), message).await;

        let long_order = CreateOrderPayload {
            user_id: "2".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(1),
            side: OrderSide::Buy,
            order_type: OrderType::MarginLong,
            leverage: Some(dec!(5)),
        };

        let message = MessageFromApi::CreateOrder { data: long_order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.margin_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.shorts.len(), 1);
        assert_eq!(orderbook.shorts[0].quantity, dec!(1)); 
        assert_eq!(orderbook.longs.len(), 0); 

        let users = engine.users.lock().await;
        let user2 = users.iter().find(|u| u.id == "2").unwrap();
        assert_eq!(user2.margin_positions.len(), 1);

        let position = &user2.margin_positions[0];
        assert_eq!(position.quantity, dec!(1));
        assert_eq!(position.avg_price, dec!(20));
        assert_eq!(position.position_type, OrderType::MarginLong);
    }

    #[tokio::test]
    async fn test_margin_requirements() {
        let mut engine = Engine::new();

        let order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(100),
            side: OrderSide::Buy,
            order_type: OrderType::MarginLong,
            leverage: Some(dec!(20)),
        };

        let message = MessageFromApi::CreateOrder { data: order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.margin_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.longs.len(), 0);
    }

    #[tokio::test]
    async fn test_margin_position_liquidation() {
        let mut engine = Engine::new();
        
        let pnl_monitor = Arc::new(PnlMonitor::new(
            engine.users.clone(),
            engine.price_service.clone()
        ));
        
        let pnl_monitor_clone = pnl_monitor.clone();
        tokio::spawn(async move {
            info!("Starting PnL monitor...");
            pnl_monitor_clone.start_monitoring().await;
        });

        let long_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "ETH_USDC".to_string(),
            price: dec!(3000),
            quantity: dec!(2),
            side: OrderSide::Buy,
            order_type: OrderType::MarginLong,
            leverage: Some(dec!(5)),
        };

        let short_order = CreateOrderPayload {
            user_id: "2".to_string(),
            market: "ETH_USDC".to_string(),
            price: dec!(3000),
            quantity: dec!(2),
            side: OrderSide::Sell,
            order_type: OrderType::MarginShort,
            leverage: Some(dec!(5)),
        };

        engine.process("test_client".to_string(), MessageFromApi::CreateOrder { data: short_order }).await;
        engine.process("test_client".to_string(), MessageFromApi::CreateOrder { data: long_order }).await;

        sleep(Duration::from_millis(100)).await;

        {
            let mut users = engine.users.lock().await;
            let user1 = users.iter_mut().find(|u| u.id == "1").unwrap();
            
            assert_eq!(user1.margin_positions.len(), 1);
            let position = &mut user1.margin_positions[0];
            assert_eq!(position.quantity, dec!(2));
            assert_eq!(position.avg_price, dec!(3000));
            assert_eq!(position.position_type, OrderType::MarginLong);

            let _margin = dec!(1200); 
            let liquidation_price = dec!(2400);
            position.liquidation_price = Some(liquidation_price);
            
            info!("Position created - Quantity: {}, Entry: {}, Liquidation: {:?}", 
                position.quantity, position.avg_price, position.liquidation_price);
        }

        info!("Updating price to trigger liquidation...");
        engine.price_service.update_trade_price("ETH_USDC", dec!(2300)).await;

        {
            let price = engine.price_service.get_price("ETH_USDC").await;
            info!("New price: {:?}", price);
            assert_eq!(price, Some(dec!(2300)), "Price should be updated to 2300");
        }

        for _ in 0..5 {
            {
                let users = engine.users.lock().await;
                let user1 = users.iter().find(|u| u.id == "1").unwrap();
                if user1.margin_positions.is_empty() {
                    info!("Position liquidated successfully");
                    break;
                }
            }
            info!("Waiting for liquidation...");
            sleep(Duration::from_secs(1)).await;
        }

        {
            let users = engine.users.lock().await;
            let user1 = users.iter().find(|u| u.id == "1").unwrap();
            
            assert_eq!(user1.margin_positions.len(), 0, "Position should be liquidated");
            
            let expected_loss = dec!(2) * (dec!(2300) - dec!(3000));
            assert_eq!(user1.realized_pnl, expected_loss, "Realized PnL should reflect liquidation loss");

            let usdc_balance = user1.balances.iter().find(|b| b.ticker == "USDC").unwrap();
            info!("Final balance state - Balance: {}, Locked: {}, PnL: {}", 
                usdc_balance.balance, usdc_balance.locked_balance, user1.realized_pnl);
        }
    }
}

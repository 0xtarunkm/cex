#[cfg(test)]
mod margin_orderbook_tests {
    use crate::{
        models::{CreateOrderPayload, MessageFromApi, OrderSide, OrderType},
        trade::Engine,
    };
    use rust_decimal_macros::dec;

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

        // Create a short order
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

        // Create a matching long order
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

        // Verify partial fill
        let orderbooks = engine.margin_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.shorts.len(), 1);
        assert_eq!(orderbook.shorts[0].quantity, dec!(1)); // Original quantity was 2, should be 1 after partial fill
        assert_eq!(orderbook.longs.len(), 0); // Long order should be fully filled

        // Verify margin positions
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

        // Try to create an order with excessive leverage
        let order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(100),
            side: OrderSide::Buy,
            order_type: OrderType::MarginLong,
            leverage: Some(dec!(20)), // Max leverage is 10
        };

        let message = MessageFromApi::CreateOrder { data: order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.margin_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.longs.len(), 0); // Order should not be placed due to excessive leverage
    }
}

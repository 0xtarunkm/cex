#[cfg(test)]
mod spot_orderbook_tests {
    use crate::{
        models::{CancelOrderPayload, CreateOrderPayload, MessageFromApi, OrderSide, OrderType},
        trade::Engine,
    };
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_create_spot_buy_order() {
        let mut engine = Engine::new();

        let order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(2),
            side: OrderSide::Buy,
            order_type: OrderType::Spot,
            leverage: None,
        };

        let message = MessageFromApi::CreateOrder { data: order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.spot_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.bids[0].price, dec!(20));
        assert_eq!(orderbook.bids[0].quantity, dec!(2));
        assert_eq!(orderbook.bids[0].user_id, "1");
    }

    #[tokio::test]
    async fn test_create_spot_sell_order() {
        let mut engine = Engine::new();

        let order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(22),
            quantity: dec!(1),
            side: OrderSide::Sell,
            order_type: OrderType::Spot,
            leverage: None,
        };

        let message = MessageFromApi::CreateOrder { data: order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.spot_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.asks.len(), 1);
        assert_eq!(orderbook.asks[0].price, dec!(22));
        assert_eq!(orderbook.asks[0].quantity, dec!(1));
        assert_eq!(orderbook.asks[0].user_id, "1");
    }

    #[tokio::test]
    async fn test_cancel_spot_order() {
        let mut engine = Engine::new();

        // First create an order
        let create_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(2),
            side: OrderSide::Buy,
            order_type: OrderType::Spot,
            leverage: None,
        };

        let message = MessageFromApi::CreateOrder { data: create_order };
        engine.process("test_client".to_string(), message).await;

        // Get the order ID
        let order_id = {
            let orderbooks = engine.spot_orderbooks.lock().await;
            let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;
            orderbook.bids[0].id.clone()
        }; // immutable borrow ends here

        // Now we can mutably borrow engine
        let cancel_order = CancelOrderPayload {
            order_id,
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
        };

        let message = MessageFromApi::CancelOrder { data: cancel_order };
        engine.process("test_client".to_string(), message).await;

        // Verify order was cancelled
        let orderbooks = engine.spot_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;
        assert_eq!(orderbook.bids.len(), 0);
    }

    #[tokio::test]
    async fn test_order_matching() {
        let mut engine = Engine::new();

        // Create a sell order first
        let sell_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(2),
            side: OrderSide::Sell,
            order_type: OrderType::Spot,
            leverage: None,
        };

        let message = MessageFromApi::CreateOrder { data: sell_order };
        engine.process("test_client".to_string(), message).await;

        // Create a matching buy order
        let buy_order = CreateOrderPayload {
            user_id: "2".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(1),
            side: OrderSide::Buy,
            order_type: OrderType::Spot,
            leverage: None,
        };

        let message = MessageFromApi::CreateOrder { data: buy_order };
        engine.process("test_client".to_string(), message).await;

        // Verify partial fill
        let orderbooks = engine.spot_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.asks.len(), 1);
        assert_eq!(orderbook.asks[0].quantity, dec!(1)); // Original quantity was 2, should be 1 after partial fill
        assert_eq!(orderbook.bids.len(), 0); // Buy order should be fully filled
    }

    #[tokio::test]
    async fn test_insufficient_balance() {
        let mut engine = Engine::new();

        // Try to buy with insufficient USDC balance
        let buy_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20000),   // Very high price
            quantity: dec!(1000), // Large quantity
            side: OrderSide::Buy,
            order_type: OrderType::Spot,
            leverage: None,
        };

        let message = MessageFromApi::CreateOrder { data: buy_order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.spot_orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.bids.len(), 0); // Order should not be placed due to insufficient balance
    }
}

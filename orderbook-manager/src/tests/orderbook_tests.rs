#[cfg(test)]
mod orderbook_tests {
    use crate::{
        models::{
            Balance, CancelOrderPayload, CreateOrderPayload, MessageFromApi, OrderSide, OrderType,
            PositionType, User,
        },
        services::price_service::PriceInfo,
        trade::Engine,
    };
    use rust_decimal_macros::dec;
    use std::time::Duration;

    #[tokio::test]
    async fn test_create_spot_buy_order() {
        let mut engine = Engine::new();

        let order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(2),
            side: OrderSide::Buy,
            is_margin: false,
            order_type: OrderType::Spot,
            leverage: Some(dec!(1)),
        };

        let message = MessageFromApi::CreateOrder { data: order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.orderbooks.lock().await;
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
            is_margin: false,
            order_type: OrderType::Spot,
            leverage: Some(dec!(1)),
        };

        let message = MessageFromApi::CreateOrder { data: order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.asks.len(), 1);
        assert_eq!(orderbook.asks[0].price, dec!(22));
        assert_eq!(orderbook.asks[0].quantity, dec!(1));
        assert_eq!(orderbook.asks[0].user_id, "1");
    }

    #[tokio::test]
    async fn test_cancel_spot_order() {
        let mut engine = Engine::new();

        let create_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(2),
            side: OrderSide::Buy,
            is_margin: false,
            order_type: OrderType::Spot,
            leverage: Some(dec!(1)),
        };

        let message = MessageFromApi::CreateOrder { data: create_order };
        engine.process("test_client".to_string(), message).await;

        let order_id = {
            let orderbooks = engine.orderbooks.lock().await;
            let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;
            orderbook.bids[0].id.clone()
        };

        let cancel_order = CancelOrderPayload {
            order_id,
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
        };

        let message = MessageFromApi::CancelOrder { data: cancel_order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;
        assert_eq!(orderbook.bids.len(), 0);
    }

    #[tokio::test]
    async fn test_order_matching() {
        let mut engine = Engine::new();

        let sell_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(2),
            side: OrderSide::Sell,
            is_margin: false,
            order_type: OrderType::Spot,
            leverage: Some(dec!(1)),
        };

        let message = MessageFromApi::CreateOrder { data: sell_order };
        engine.process("test_client".to_string(), message).await;

        let buy_order = CreateOrderPayload {
            user_id: "2".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(1),
            side: OrderSide::Buy,
            is_margin: false,
            order_type: OrderType::Spot,
            leverage: Some(dec!(1)),
        };

        let message = MessageFromApi::CreateOrder { data: buy_order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.asks.len(), 1);
        assert_eq!(orderbook.asks[0].quantity, dec!(1));
        assert_eq!(orderbook.bids.len(), 0);
    }

    #[tokio::test]
    async fn test_insufficient_balance() {
        let mut engine = Engine::new();

        let buy_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20000),
            quantity: dec!(1000),
            side: OrderSide::Buy,
            is_margin: false,
            order_type: OrderType::Spot,
            leverage: Some(dec!(1)),
        };

        let message = MessageFromApi::CreateOrder { data: buy_order };
        engine.process("test_client".to_string(), message).await;

        let orderbooks = engine.orderbooks.lock().await;
        let orderbook = orderbooks.get("SOL_USDC").unwrap().lock().await;

        assert_eq!(orderbook.bids.len(), 0);
    }

    #[tokio::test]
    async fn test_margin_long_position() {
        let mut engine = Engine::new();

        {
            let mut users = engine.users.write().await;
            users.push(User {
                id: "1".to_string(),
                balances: vec![
                    Balance {
                        ticker: "USDC".to_string(),
                        balance: dec!(100000),
                        locked_balance: dec!(0),
                    },
                    Balance {
                        ticker: "SOL".to_string(),
                        balance: dec!(100000),
                        locked_balance: dec!(0),
                    },
                ],
                margin_enabled: true,
                margin_positions: Vec::new(),
                max_leverage: dec!(10),
                margin_used: dec!(0),
                realized_pnl: dec!(0),
            });

            // Add counter-party user
            users.push(User {
                id: "2".to_string(),
                balances: vec![
                    Balance {
                        ticker: "USDC".to_string(),
                        balance: dec!(100000),
                        locked_balance: dec!(0),
                    },
                    Balance {
                        ticker: "SOL".to_string(),
                        balance: dec!(100000),
                        locked_balance: dec!(0),
                    },
                ],
                margin_enabled: true,
                margin_positions: Vec::new(),
                max_leverage: dec!(10),
                margin_used: dec!(0),
                realized_pnl: dec!(0),
            });
        }

        // Create a matching sell order first
        let sell_order = CreateOrderPayload {
            user_id: "2".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(5),
            side: OrderSide::Sell,
            is_margin: false,
            order_type: OrderType::Spot,
            leverage: Some(dec!(1)),
        };

        let message = MessageFromApi::CreateOrder { data: sell_order };
        engine.process("test_client".to_string(), message).await;

        // Then create the margin long order
        let buy_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(5),
            side: OrderSide::Buy,
            is_margin: true,
            order_type: OrderType::MarginLong,
            leverage: Some(dec!(5)),
        };

        let message = MessageFromApi::CreateOrder { data: buy_order };
        engine.process("test_client".to_string(), message).await;

        let users = engine.users.read().await;
        let user = users.iter().find(|u| u.id == "1").unwrap();

        assert_eq!(user.margin_positions.len(), 1, "Should have one position");
        let position = &user.margin_positions[0];
        assert_eq!(position.asset, "SOL_USDC", "Asset should be SOL_USDC");
        assert_eq!(position.size, dec!(5), "Size should be 5");
        assert_eq!(position.entry_price, dec!(20), "Entry price should be 20");
        assert_eq!(
            position.position_type,
            PositionType::Long,
            "Should be a long position"
        );
    }

    #[tokio::test]
    async fn test_margin_short_position() {
        let mut engine = Engine::new();

        {
            let mut users = engine.users.write().await;
            users.push(User {
                id: "1".to_string(),
                balances: vec![
                    Balance {
                        ticker: "USDC".to_string(),
                        balance: dec!(100000),
                        locked_balance: dec!(0),
                    },
                    Balance {
                        ticker: "SOL".to_string(),
                        balance: dec!(100),
                        locked_balance: dec!(0),
                    },
                ],
                margin_enabled: true,
                margin_positions: Vec::new(),
                max_leverage: dec!(10),
                margin_used: dec!(0),
                realized_pnl: dec!(0),
            });

            // Add counter-party user
            users.push(User {
                id: "2".to_string(),
                balances: vec![
                    Balance {
                        ticker: "USDC".to_string(),
                        balance: dec!(100000),
                        locked_balance: dec!(0),
                    },
                    Balance {
                        ticker: "SOL".to_string(),
                        balance: dec!(100000),
                        locked_balance: dec!(0),
                    },
                ],
                margin_enabled: true,
                margin_positions: Vec::new(),
                max_leverage: dec!(10),
                margin_used: dec!(0),
                realized_pnl: dec!(0),
            });
        }

        // Create a matching buy order first
        let buy_order = CreateOrderPayload {
            user_id: "2".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(5),
            side: OrderSide::Buy,
            is_margin: false,
            order_type: OrderType::Spot,
            leverage: Some(dec!(1)),
        };

        let message = MessageFromApi::CreateOrder { data: buy_order };
        engine.process("test_client".to_string(), message).await;

        // Then create the margin short order
        let short_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(20),
            quantity: dec!(5),
            side: OrderSide::Sell,
            is_margin: true,
            order_type: OrderType::MarginShort,
            leverage: Some(dec!(5)),
        };

        let message = MessageFromApi::CreateOrder { data: short_order };
        engine.process("test_client".to_string(), message).await;

        let users = engine.users.read().await;
        let user = users.iter().find(|u| u.id == "1").unwrap();

        assert_eq!(user.margin_positions.len(), 1, "Should have one position");
        let position = &user.margin_positions[0];
        assert_eq!(position.asset, "SOL_USDC", "Asset should be SOL_USDC");
        assert_eq!(position.size, dec!(5), "Size should be 5");
        assert_eq!(position.entry_price, dec!(20), "Entry price should be 20");
        assert_eq!(
            position.position_type,
            PositionType::Short,
            "Should be a short position"
        );
    }

    #[tokio::test]
    async fn test_liquidation() {
        let mut engine = Engine::new();

        {
            let mut users = engine.users.write().await;
            users.push(User {
                id: "1".to_string(),
                balances: vec![
                    Balance {
                        ticker: "USDC".to_string(),
                        balance: dec!(100000),
                        locked_balance: dec!(0),
                    },
                    Balance {
                        ticker: "SOL".to_string(),
                        balance: dec!(100000),
                        locked_balance: dec!(0),
                    },
                ],
                margin_enabled: true,
                margin_positions: Vec::new(),
                max_leverage: dec!(10),
                margin_used: dec!(0),
                realized_pnl: dec!(0),
            });
        }

        let create_order = CreateOrderPayload {
            user_id: "1".to_string(),
            market: "SOL_USDC".to_string(),
            price: dec!(100),
            quantity: dec!(1),
            side: OrderSide::Buy,
            is_margin: true,
            order_type: OrderType::MarginLong,
            leverage: Some(dec!(5)),
        };

        let message = MessageFromApi::CreateOrder { data: create_order };
        engine.process("test_client".to_string(), message).await;

        let price_info = PriceInfo {
            last_trade_price: Some(dec!(50)),
            mark_price: dec!(50),
            index_price: Some(dec!(50)),
            timestamp: chrono::Utc::now().timestamp(),
        };

        engine
            .price_service
            .update_price("SOL_USDC", price_info)
            .await;

        tokio::time::sleep(Duration::from_secs(2)).await;

        let users = engine.users.read().await;
        let user = users.iter().find(|u| u.id == "1").unwrap();

        assert_eq!(
            user.margin_positions.len(),
            0,
            "Position should be liquidated"
        );
    }
}

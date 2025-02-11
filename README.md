
# CEX (Centralized Exchange)

A high-performance centralized cryptocurrency exchange implementation in Rust, featuring spot and margin trading capabilities with real-time market data distribution.

## 🏗 Architecture
<img width="1064" alt="Screenshot 2025-02-11 at 5 50 06 PM" src="https://github.com/user-attachments/assets/6b31f943-ba80-435c-b7f7-7f06b9d34259" />

The project consists of three main components:

1. **HTTP Server** (`http-server/`):
   - RESTful API endpoints for order management
   - Built with Axum framework
   - Redis-based communication with the order book manager

2. **Orderbook Manager** (`orderbook-manager/`):
   - Core trading engine implementation
   - Real-time order matching
   - Position and PnL management
   - Price service for mark and index prices
   - Market depth maintenance

3. **WebSocket Server** (`websocket-server/`):
   - Real-time market data streaming
   - Room-based subscription system
   - Redis pub/sub integration
   - Client connection management
  
4. **Wallet Manager** (`wallet-manager/`):
   - Wallet management
   - Balance tracking
   - Transaction history
   - Onramp and offramp support

## 🚀 Features

- **Trading Capabilities**:
  - Spot trading with limit orders
  - Margin trading with up to 10x leverage
  - Real-time order matching
  - Position tracking and PnL monitoring

- **Market Data**:
  - Real-time orderbook depth
  - Trade execution broadcasts
  - Price updates (mark, index, last)
  - WebSocket streaming

- **Risk Management**:
  - Margin requirement validation
  - Position monitoring
  - Liquidation price calculation
  - Balance checks

## 💡 API Access

### REST API Endpoints
Base URL: `http://localhost:8080/api/v1`

#### Order Operations
- `POST /order/create` - Create new order
- `DELETE /order/delete` - Cancel existing order
- `GET /order/open/{user_id}/{market}` - Get user's open orders
- `POST /order/quote` - Get quote for potential trade
- `GET /order/margin_positions/{user_id}` - Get margin positions

#### User Operations
- `GET /user/balances/{user_id}` - Get user balances
- `POST /user/onramp` - Handle user deposits

#### Market Data
- `GET /depth/{market}/{order_type}` - Get order book depth
- `GET /ticker/{market}/{order_type}` - Get market ticker

### WebSocket API
Server URL: `ws://localhost:8081`

#### Market Data Channels
- `SOL_USDC` - SOL/USDC market
- `BTC_USDC` - BTC/USDC market
- `ETH_USDC` - ETH/USDC market

#### Message Types

- `SUBSCRIBE` - Subscribe to a market
- `UNSUBSCRIBE` - Unsubscribe from a market

#### Market Data Channels

- `trade` - Trade executions
- `depth` - Order book depth
- `ticker` - Market ticker

## 🏃‍♂️ Getting Started

1. Prerequisites:
   - Rust toolchain
   - Docker and Docker Compose
   - Redis server

2. Start Dependencies:

```bash
docker compose up -d
```

3. Run the server:

```bash
cargo run --bin http-server
```

4. Run the websocket server:

```bash
cargo run --bin websocket-server
```

5. Run the orderbook manager:

```bash
cargo run --bin orderbook-manager
```

6. Run the wallet manager:

```bash
cargo run --bin wallet-manager
```


## 📊 Trading Flows

### Spot Trading
1. Balance validation
2. Order matching
3. Real-time execution
4. Balance updates
5. WebSocket notifications

### Margin Trading
1. Margin requirement checks
2. Position creation/update
3. PnL monitoring
4. Liquidation price tracking
5. Real-time position updates

## 🔧 Implementation Details

### Data Structures
- Priority queue-based orderbooks
- Position tracking system
- Real-time price service
- WebSocket subscription management

### Communication
- Redis for inter-service messaging
- WebSocket for real-time updates
- REST API for order management

### Monitoring
- Continuous PnL calculation
- Position risk assessment
- Balance tracking
- Market data broadcasting

## 📈 Development Status

This is a work in progress with core functionality implemented:
- ✅ Order matching engine
- ✅ Margin trading support
- ✅ Real-time market data
- ✅ WebSocket streaming
- ✅ Basic risk management

---
Built with 🦀 Rust and ❤️




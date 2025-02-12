
# CEX (Centralized Exchange)

A high-performance centralized cryptocurrency exchange implementation in Rust, featuring spot and margin trading capabilities with real-time market data distribution.

## Architecture
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

## Features

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

## API Access
[Postman Collection](https://www.postman.com/solar-trinity-740656/cex/overview)

## Setup

```bash
docker compose up --build -d
```
yes that's it!

## Trading Flows

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

# Implementation Details

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

---
Built with 🦀 and ❤️

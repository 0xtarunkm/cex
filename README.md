
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

## Setup

```bash
docker compose up --build -d
```

## API Access
[Postman Collection](https://www.postman.com/solar-trinity-740656/cex/overview)

---
Built with 🦀 and ❤️

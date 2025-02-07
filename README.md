# CEX (Centralized Exchange)

A high-performance centralized cryptocurrency exchange implementation in Rust, featuring spot and margin trading capabilities.

## Features
![Screenshot 2025-01-23 at 7 01 12 PM](https://github.com/user-attachments/assets/b2e2a138-2a9b-44ce-b073-3252a4edd8fa)

- **Trading Pairs**: Support for multiple trading pairs (SOL/USDC, BTC/USDC, ETH/USDC)
- **Order Types**:
  - Spot trading
  - Margin trading (Long/Short positions)
  - Support for leverage up to 10x
- **Order Book Management**:
  - Real-time order matching engine
  - Market depth tracking
  - Quote generation
- **User Management**:
  - Balance tracking
  - Position management
  - Margin account support

## Architecture

The project consists of two main components:

1. **HTTP Server** (`http-server/`):
   - RESTful API endpoints for order management
   - Built with Actix-web framework
   - Redis-based communication with the order book manager

2. **Orderbook Manager** (`orderbook-manager/`):
   - Core trading engine implementation
   - Real-time order matching
   - Balance and position management
   - Market depth maintenance

## API Endpoints

### Order Management
- `POST /api/v1/order/create` - Create new order
- `DELETE /api/v1/order/delete` - Cancel existing order
- `GET /api/v1/order/get_quote` - Get quote for potential trade
- `GET /api/v1/order/open` - Get user's open orders

### Market Data
- `POST /api/v1/depth` - Get order book depth

## Setup

1. Prerequisites:
   - Rust toolchain
   - Docker and Docker Compose
   - Redis server

2. Start Dependencies:
   ```bash
   # Start Redis and database services
   docker compose up -d
   ```

3. Start Services:
   ```bash
   # Start the orderbook manager
   cd orderbook-manager
   cargo run

   # In a new terminal, start the HTTP server
   cd http-server
   cargo run
   ```

The HTTP server will be available at `127.0.0.1:8080` by default.

## Implementation Details

- **Concurrent Order Processing**: Uses Rust's mutex and Arc for thread-safe order book management
- **Real-time Communication**: Redis-based message passing between components
- **Market Safety**: Implements balance checks and margin requirements validation
- **Position Management**: Tracks user positions and enforces leverage limits

## Development Status

This is a work in progress. Current implementation includes core trading functionality with spot and margin trading support.
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

## Trading Flows

### Spot Trading
1. **Balance Validation**:
   - For Buy orders: Checks if user has sufficient USDC (price * quantity)
   - For Sell orders: Checks if user has sufficient base asset (e.g., SOL)

2. **Order Processing**:
   - Locks the required balance
   - Matches against existing orders in the orderbook
   - If partially filled, places remaining amount in orderbook
   - Updates user balances after successful trades

### Margin Trading
1. **Margin Requirements Validation**:
   - Verifies user has margin trading enabled
   - Checks if leverage is within limits (max 10x)
   - Validates required margin: (price * quantity) / leverage
   - For shorts: Additional 10% safety margin is required

2. **Long Position Flow**:
   - Requires initial margin: (price * quantity) / leverage
   - Locks the required margin amount
   - Creates or updates existing long position
   - Calculates liquidation price based on entry price and leverage
   - Tracks unrealized PnL

3. **Short Position Flow**:
   - Requires initial margin with safety multiplier
   - Locks the required margin amount
   - Creates or updates existing short position
   - Calculates liquidation price based on entry price and leverage
   - Credits user with borrowed asset value
   - Tracks unrealized PnL

4. **Position Management**:
   - Continuously updates unrealized PnL
   - Monitors for liquidation price breaches
   - Allows position closure through opposite orders
   - Updates realized PnL on position closure

5. **Liquidation**:
   - Triggers when market price crosses liquidation threshold
   - Liquidation price = Entry price ± (Entry price * Liquidation threshold / Leverage)
   - For longs: Liquidation below entry price
   - For shorts: Liquidation above entry price

## Development Status

This is a work in progress. Current implementation includes core trading functionality with spot and margin trading support.
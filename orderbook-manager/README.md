# 🏦 Orderbook Manager

A high-performance trading engine written in Rust that manages order matching, execution, and position tracking for both spot and margin trading.

## 🚀 Features

- Real-time order matching engine
- Support for both spot and margin trading
- Position tracking and PnL monitoring
- Price service for mark and index prices
- Redis-based communication system
- Multiple market support (SOL/USDC, BTC/USDC, ETH/USDC)

## 🏗 Architecture

The orderbook manager runs as a standalone service that:
1. Listens for incoming orders via Redis
2. Processes orders through the matching engine
3. Updates user balances and positions
4. Broadcasts market data updates

## 💡 Supported Operations

### Order Types
- Spot orders (buy/sell)
- Margin long positions
- Margin short positions

### Market Data
- Real-time orderbook depth
- Price updates
- Trade execution broadcasts

### Risk Management
- Margin requirement validation
- Balance checks
- Position tracking
- Liquidation price monitoring

## 🔧 Technical Details

### Dependencies
- Tokio for async runtime
- Redis for message queue
- Serde for serialization
- Decimal for precise calculations

### Market Data Channels
- `orderbook_channel`: Orderbook updates
- `price_channel`: Price updates
- `trade_channel`: Trade execution updates

### Redis Channels

- `MESSAGE_FROM_API_CHANNEL`: Incoming orders from the API
- `MESSAGE_TO_API_CHANNEL`: Outgoing order confirmations and updates
- `PRICE_CHANNEL`: Price updates from the price service
- `TRADE_CHANNEL`: Trade execution updates

## 📊 Market Structure

Each market maintains two separate orderbooks:
- Spot orderbook (bids and asks)
- Margin orderbook (longs and shorts)

### Order Storage & Matching

#### Spot Orders
- Stored in two priority queues (implemented as sorted vectors):
  - Bids: Sorted by highest price first, then earliest timestamp
  - Asks: Sorted by lowest price first, then earliest timestamp
- Matching Algorithm:
  1. Buy orders match against lowest asks
  2. Sell orders match against highest bids
  3. Orders at same price level execute in FIFO order

#### Margin Orders
- Similar structure but separated into longs and shorts:
  - Longs: Sorted by highest price first (like bids)
  - Shorts: Sorted by lowest price first (like asks)
- Additional tracking for:
  - Position size
  - Entry price
  - Leverage used
  - Liquidation prices

### Price Service

The price service maintains three key prices for each market:
1. Last Trade Price: Price of most recent execution
2. Mark Price: Used for PnL calculations and liquidations
   - Calculated as: `(best_bid + best_ask) / 2`
3. Index Price: External reference price (if available)

Price updates are broadcast on dedicated Redis channels:


### PnL Monitoring

The PnL service runs as a background task that:
1. Monitors all open margin positions
2. Calculates unrealized PnL using mark price:
   ```rust
   // For longs:
   unrealized_pnl = (current_mark_price - entry_price) * position_size
   
   // For shorts:
   unrealized_pnl = (entry_price - current_mark_price) * position_size
   ```
3. Checks liquidation conditions:
   - Position value falls below maintenance margin
   - Unrealized losses exceed available margin
4. Updates user margin usage:
   ```rust
   margin_used = position_value / leverage
   ```

### Risk Calculations

For margin positions:
- Initial Margin = Position Value / Leverage
- Maintenance Margin = Initial Margin * Maintenance Factor
- Liquidation Price calculated as:
  ```rust
  // For longs:
  liquidation_price = entry_price * (1 - 1/leverage + maintenance_margin)
  
  // For shorts:
  liquidation_price = entry_price * (1 + 1/leverage - maintenance_margin)
  ```

## 🔒 Risk Controls

- Maximum leverage limits
- Margin requirement validation
- Balance verification
- Position size limits
- Continuous PnL monitoring

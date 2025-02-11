# 🚀 http-server

A super simple HTTP server built with Rust and Axum for handling trading operations.

## What's this?

This is a trading-focused HTTP server that provides endpoints for order management, user balances, market depth, and more. It's built for speed and reliability using Rust! 🦀


## API Routes

All routes are prefixed with `/api/v1`

### Health Check
- `GET /healthcheck` - Check if the server is alive

### Order Operations
- `POST /order/create` - Create a new order
- `DELETE /order/delete` - Cancel an existing order
- `GET /order/open/{user_id}/{market}` - Get all open orders for a user in a specific market
- `POST /order/quote` - Get a quote for an order
- `GET /order/margin_positions/{user_id}` - Get margin positions for a user

### User Operations
- `GET /user/balances/{user_id}` - Get user balances
- `POST /user/onramp` - Handle user onramp operations

### Market Data
- `GET /depth/{market}/{order_type}` - Get market depth
- `GET /ticker/{market}/{order_type}` - Get market ticker information

---

Built with 🦀 Rust and ❤️
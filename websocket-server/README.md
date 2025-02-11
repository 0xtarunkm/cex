# 🔌 WebSocket Server

A high-performance WebSocket server written in Rust that handles real-time market data distribution and user subscriptions.

## 🚀 Features

- Real-time market data streaming
- Room-based subscription system
- Redis pub/sub integration
- Automatic ping/pong heartbeat
- Connection management

## 📡 Connection Details

- Server runs on: `ws://0.0.0.0:8081`
- Heartbeat interval: 30 seconds

## 📨 Message Types

### Subscribe

```json
{
  "type": "SUBSCRIBE",
  "room": "SOL_USDC"
}

### Unsubscribe

```json
{
  "type": "UNSUBSCRIBE",
  "room": "SOL_USDC"
}

### Send Message

```json
{
  "type": "SEND_MESSAGE",
  "room": "SOL_USDC",
  "message": "Hello, world!"
}
```


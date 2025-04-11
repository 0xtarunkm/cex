// import Decimal from 'decimal.js'; // Remove import for now

// Based on message_from_engine.rs

export type OrderSide = 'BUY' | 'SELL';

export interface OrderDetails {
  type_: OrderSide;
  quantity: string; // Use string for simplicity
}

// Representing a single level in the order book for UI display
export interface OrderBookLevel {
    price: string; // Use string for display formatting
    size: string; // Use string for display formatting
    total: string; // Use string for display formatting
    isBid: boolean;
}

// Simplified Trade structure for UI
export interface Trade {
    id: string;
    price: string; // Use string for display formatting
    size: string; // Use string for display formatting
    side: OrderSide;
    timestamp: number; // Unix timestamp
}

// Standard Kline/Candlestick structure
export interface Kline {
    timestamp: number; // Start time of the interval
    open: string;
    high: string;
    low: string;
    close: string;
    volume: string;
}

export interface Balance {
    asset: string; // e.g., "USDC", "SOL"
    ticker: string; // e.g., "USD Coin", "Solana"
    icon?: string; // Optional: Path or component for the icon
    balance: string; // Total balance (formatted)
    balanceValue: string; // USD value of total balance (formatted)
    available: string; // Available balance (formatted)
    availableValue: string; // USD value of available balance (formatted)
    lendBorrow: string | null; // Lend/Borrow amount or null/'-'
    openOrders: string; // Open orders amount (formatted)
} 
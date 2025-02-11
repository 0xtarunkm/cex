-- Add up migration script here
CREATE TABLE user_accounts IF NOT EXISTS (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    solana_public_key TEXT NOT NULL,
    solana_private_key TEXT NOT NULL,
    solana_balance DECIMAL(18, 8) NOT NULL,
    usdc_balance DECIMAL(18, 8) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

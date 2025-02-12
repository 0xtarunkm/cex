-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    solana_public_key TEXT NOT NULL,
    solana_private_key TEXT NOT NULL,
    balance DECIMAL(18, 8) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

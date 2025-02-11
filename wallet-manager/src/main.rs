use std::collections::HashMap;

use anyhow::Result;
use constants::ONRAMP_CHANNEL;
use redis::Commands;
use services::{account_monitor::GrpcStreamManager, redis_manager::RedisManager};
use solana_sdk::{signature::Keypair, signer::Signer};
use tracing::{error, info};
use yellowstone_grpc_proto::geyser::{CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts};

use serde_json;
use serde::Deserialize;

mod constants;
mod services;

#[derive(Debug, Deserialize)]
struct IncomingMessage {
    client_id: String,
    message: WalletMessage,
}

#[derive(Debug, Deserialize)]
struct WalletMessage {
    network: String,
    token: String,
    user_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut users: Vec<String> = Vec::new();

    let manager = GrpcStreamManager::new(
        "your-grpc-url:2053",
        "your-x-token",
    ).await?;

    let mut manager_lock = manager.lock().await;

    let request = SubscribeRequest {
        accounts: HashMap::from_iter(vec![(
            "client".to_string(),
            SubscribeRequestFilterAccounts {
                account: users.clone(),
                owner: vec![],
                filters: vec![],
                nonempty_txn_signature: Some(false),
            },
        )]),
        commitment: Some(CommitmentLevel::Confirmed as i32),
        ..Default::default()
    };

    let result = manager_lock.connect(request).await;
    if let Err(e) = &result {
        error!("Subscription error: {:?}", e);
    }
    result?;
    info!("Wallet manager started");

    let redis_manager = RedisManager::instance();
    let mut conn = redis_manager.get_connection()?;

    loop {
        let response: Option<(String, String)> = conn.brpop(ONRAMP_CHANNEL, 0.0)?;

        match response {
            Some((_, message)) => {
                info!("Received message: {}", message);
                match serde_json::from_str::<IncomingMessage>(&message) {
                    Ok(parsed_message) => {
                        let keypair = Keypair::new();
                        let public_key = keypair.pubkey().to_string();
                        users.push(public_key.clone());

                        info!("Sending public key to client: {}", public_key);
                        redis_manager.send_to_api(&parsed_message.client_id, &public_key)?;
                    },
                    Err(e) => {
                        error!("Failed to parse message: {}. Message was: {}", e, message);
                    }
                }
            }
            None => {}
        }
    }
}

use std::collections::HashMap;
use anyhow::Result;
use constants::ONRAMP_CHANNEL;
use db::{db_connection, db_queries};
use dotenv::dotenv;
use models::IncomingMessage;
use redis::Commands;
use services::{
    account_monitor::GrpcStreamManager, redis_manager::RedisManager, wallet_service::WalletService,
};
use tracing::{error, info};
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts,
};
use serde_json;


mod constants;
mod models;
mod services;
mod db;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

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

    let mut wallet_service = WalletService::new();

    loop {
        let response: Option<(String, String)> = conn.brpop(ONRAMP_CHANNEL, 0.0)?;

        match response {
            Some((_, message)) => {
                match serde_json::from_str::<IncomingMessage>(&message) {
                    Ok(parsed_message) => {
                        let pool = db_connection::get_connection_pool().await?;
                        
                        let (public_key, private_key) = wallet_service.generate_wallet()?;
                        users.push(public_key.clone());

                        db_queries::create_user_wallet(
                            &pool,
                            parsed_message.user_id,
                            &public_key,
                            &private_key
                        ).await?;

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

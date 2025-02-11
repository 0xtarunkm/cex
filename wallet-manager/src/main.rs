use std::collections::HashMap;

use anyhow::Result;
use services::account_monitor::GrpcStreamManager;
use tracing::error;
use yellowstone_grpc_proto::geyser::{CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts};

mod services;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = GrpcStreamManager::new(
        "your-grpc-url:2053",
        "your-x-token",
    ).await?;

    let mut manager_lock = manager.lock().await;

    let request = SubscribeRequest {
        accounts: HashMap::from_iter(vec![(
            "client".to_string(),
            SubscribeRequestFilterAccounts {
                account: vec!["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()],
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

    Ok(())
}

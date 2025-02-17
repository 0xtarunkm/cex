use std::{sync::Arc, time::Duration};

use crate::db::{db_connection, db_queries};
use anyhow::Result;
use bigdecimal::BigDecimal;
use futures::{SinkExt, StreamExt};
use solana_sdk::pubkey::Pubkey;
use tokio::sync::Mutex;
use tonic::{metadata::errors::InvalidMetadataValue, transport::Endpoint};
use tonic_health::pb::health_client::HealthClient;
use tracing::{error, info};
use yellowstone_grpc_client::{GeyserGrpcClient, InterceptorXToken};
use yellowstone_grpc_proto::geyser::{
    geyser_client::GeyserClient, subscribe_update::UpdateOneof, SubscribeRequest,
    SubscribeRequestPing,
};

pub struct GrpcStreamManager {
    client: GeyserGrpcClient<InterceptorXToken>,
    is_connected: bool,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    reconnect_interval: Duration,
}

impl GrpcStreamManager {
    pub async fn handle_account_update(
        &self,
        slot: u64,
        account_info: &yellowstone_grpc_proto::geyser::SubscribeUpdateAccountInfo,
    ) -> Result<()> {
        let pool = db_connection::get_connection_pool().await?;

        let pubkey = Pubkey::try_from(account_info.pubkey.clone())
            .expect("valid pubkey")
            .to_string();

        let lamports = BigDecimal::from(account_info.lamports);

        db_queries::update_solana_balance(&pool, &pubkey, lamports).await?;

        info!(
        "ACCOUNT UPDATE RECEIVED:\nSlot: {}\nPubkey: {}\nLamports: {}\nOwner: {}\nData length: {}\nExecutable: {}\nWrite version: {}\n",
            slot,
            Pubkey::try_from(account_info.pubkey.clone()).expect("valid pubkey"),
            account_info.lamports,
            Pubkey::try_from(account_info.owner.clone()).expect("valid pubkey"),
            account_info.data.len(),
            account_info.executable,
            account_info.write_version
        );

        if !account_info.data.is_empty() {
            info!("Data (hex): {}", hex::encode(&account_info.data));
        }
        Ok(())
    }

    pub async fn new(endpoint: &str, x_token: &str) -> Result<Arc<Mutex<GrpcStreamManager>>> {
        let interceptor = InterceptorXToken {
            x_token: Some(
                x_token
                    .parse()
                    .map_err(|e: InvalidMetadataValue| anyhow::Error::from(e))?,
            ),
            x_request_snapshot: true,
        };

        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(10))
            .connect()
            .await
            .map_err(|e| anyhow::Error::from(e))?;

        let client = GeyserGrpcClient::new(
            HealthClient::with_interceptor(channel.clone(), interceptor.clone()),
            GeyserClient::with_interceptor(channel, interceptor),
        );

        Ok(Arc::new(Mutex::new(GrpcStreamManager {
            client,
            is_connected: false,
            reconnect_attempts: 0,
            max_reconnect_attempts: 10,
            reconnect_interval: Duration::from_secs(5),
        })))
    }

    pub async fn connect(&mut self, request: SubscribeRequest) -> Result<()> {
        let request = request.clone();
        let (mut subscribe_tx, mut stream) = self
            .client
            .subscribe_with_request(Some(request.clone()))
            .await?;

        self.is_connected = true;
        self.reconnect_attempts = 0;

        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    match msg.update_oneof {
                        Some(UpdateOneof::Account(account)) => {
                            if let Some(account_info) = account.account {
                                self.handle_account_update(account.slot, &account_info)
                                    .await?;
                            }
                        }
                        Some(UpdateOneof::Ping(_)) => {
                            subscribe_tx
                                .send(SubscribeRequest {
                                    ping: Some(SubscribeRequestPing { id: 1 }),
                                    ..Default::default()
                                })
                                .await?;
                        }
                        Some(UpdateOneof::Pong(_)) => {} // Ignore pong responses
                        _ => {
                            println!("Other update received: {:?}", msg);
                        }
                    }
                }
                Err(err) => {
                    error!("Error: {:?}", err);
                    self.is_connected = false;
                    Box::pin(self.reconnect(request.clone())).await?;
                    break;
                }
            }
        }
        Ok(())
    }

    pub async fn reconnect(&mut self, request: SubscribeRequest) -> Result<()> {
        if self.reconnect_attempts >= self.max_reconnect_attempts {
            println!("Max reconnection attempts reached");
            return Ok(());
        }

        self.reconnect_attempts += 1;
        println!("Reconnecting... Attempt {}", self.reconnect_attempts);

        let backoff = self.reconnect_interval * std::cmp::min(self.reconnect_attempts, 5);
        tokio::time::sleep(backoff).await;

        Box::pin(self.connect(request)).await
    }
}

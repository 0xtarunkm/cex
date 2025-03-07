use crate::models::IncomingMessage;
use anyhow::Result;
use constants::MESSAGE_FROM_API_CHANNEL;
use redis::Commands;
use services::redis_manager::RedisManager;
use trade::Engine;

mod constants;
mod models;
mod services;
mod tests;
mod trade;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut engine = Engine::new();

    let redis_manager = RedisManager::instance();
    let mut conn = redis_manager.get_connection()?;

    loop {
        let response: Option<(String, String)> = conn.brpop(MESSAGE_FROM_API_CHANNEL, 0.0)?;

        match response {
            Some((_, message)) => {
                let parsed_message: IncomingMessage = serde_json::from_str(&message)?;
                engine
                    .process(parsed_message.client_id, parsed_message.message)
                    .await;
            }
            None => {}
        }
    }
}

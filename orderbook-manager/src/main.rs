use models::IncomingMessage;
use redis::Commands;
use std::time::Duration;
use tokio::time::sleep;
use trade::Engine;
use utils::redis_manager::RedisManager;

mod models;
mod trade;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new();

    let redis_manager = RedisManager::instance();
    let mut conn = redis_manager.get_connection()?;

    loop {
        let response: Option<(String, String)> = conn.brpop("messages", 2.0)?;

        match response {
            Some((_, message)) => {
                let parsed_message: IncomingMessage = serde_json::from_str(&message)?;

                println!("{}", parsed_message.client_id);
                println!("{:?}", parsed_message.message);
                engine.process(parsed_message.client_id, parsed_message.message);
            }
            None => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

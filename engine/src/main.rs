use redis::Commands;
use serde_json::Value;
use std::error::Error;
use tokio;
use trade::{Engine, MessageFromApi};

mod models;
mod trade;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut conn = client.get_connection().unwrap();

    // Declare `engine` as mutable
    let mut engine = Engine::new();

    loop {
        let response: Option<String> = conn.rpop("messages", None)?;

        println!("{:?}", response);
        if let Some(msg) = response {
            if let Ok(full_msg) = serde_json::from_str::<Value>(&msg) {
                let client_id = full_msg["client_id"].as_str().unwrap_or("").to_string();

                if let Some(message_value) = full_msg.get("message") {
                    if let Ok(message) =
                        serde_json::from_value::<MessageFromApi>(message_value.clone())
                    {
                        println!("{client_id} : {:?}", message);
                        engine.process(message, client_id);
                    } else {
                        eprintln!("Failed to deserialize inner message: {}", message_value);
                    }
                } else {
                    eprintln!("Missing 'message' field in Redis message: {}", msg);
                }
            } else {
                eprintln!("Failed to deserialize Redis message: {}", msg);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}

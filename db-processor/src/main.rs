mod db;
mod models;

use db::{connection::init_db, queries::insert_trade_price};
use models::messages::MessageFromEngine;
use redis::Commands;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let redis_manager = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut conn = redis_manager.get_connection()?;
    let connection_pool = init_db().await?;

    loop {
        let response: Option<(String, String)> = conn.brpop("db_processor", 2.0)?;

        match response {
            Some((_, message)) => {
                let parsed_message: MessageFromEngine = serde_json::from_str(&message)?;

                match parsed_message {
                    MessageFromEngine::TradeAdded { data } => {
                        insert_trade_price(&connection_pool, data.price, data.timestamp).await?;
                    }
                }
            }
            None => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

use redis::Commands;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_manager = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut conn = redis_manager.get_connection()?;

    loop {
        let response: Option<(String, String)> = conn.brpop("db_processor", 2.0)?;

        match response {
            Some((_, message)) => {
                // let parsed_message: IncomingMessage = serde_json::from_str(&message)?;

                println!("{}", message);
                // println!("{}", parsed_message.client_id);
                // println!("{:?}", parsed_message.message);
            }
            None => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

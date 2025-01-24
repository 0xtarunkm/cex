mod engine;
mod messages;
mod models;
mod orderbook;

use engine::Engine;
use messages::MessageFromApi;
use rust_decimal::Decimal;
use tokio;

#[tokio::main]
async fn main() {
    let engine = Engine::new();

    // Example usage
    let message = MessageFromApi::CreateOrder {
        market: "TATA_INR".to_string(),
        price: Decimal::from(100),
        quantity: Decimal::from(10),
        side: models::Side::Buy,
        user_id: "1".to_string(),
    };

    engine.process(message, "client_1".to_string()).await;
}

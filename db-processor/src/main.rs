use anyhow::Result;
use dotenv::dotenv;
use models::IncomingMessage;
use redis::Commands;
use services::redis_manager::RedisManager;
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::info;

mod models;
mod services;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let redis_manager = RedisManager::instance();
    let mut conn = redis_manager.get_connection()?;
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    loop {
        let response: Option<(String, String)> = conn.brpop("db_processor", 0.0)?;

        info!("Response: {:?}", response);
        match response {
            Some((_, message)) => {
                let parsed_message: IncomingMessage = serde_json::from_str(&message)?;
                info!("Adding trade data: {:?}", parsed_message);

                if parsed_message.message_type == "TRADE_ADDED" {
                    let data = parsed_message.data;
                    info!(
                        "Adding trade data: price={}, time={}",
                        data.price, data.time
                    );

                    match data.ticker.as_str() {
                        "SOL_USDC" => {
                            sqlx::query!(
                                "INSERT INTO sol_prices (time, price, currency_code) VALUES ($1, $2, $3)", 
                                OffsetDateTime::from_unix_timestamp(data.time.timestamp()).unwrap(),
                                data.price.to_string().parse::<f64>().unwrap(),
                                "SOL"
                            ).execute(&pool).await?;
                        }
                        "BTC_USDC" => {
                            sqlx::query!(
                                "INSERT INTO btc_prices (time, price, currency_code) VALUES ($1, $2, $3)", 
                                OffsetDateTime::from_unix_timestamp(data.time.timestamp()).unwrap(),
                                data.price.to_string().parse::<f64>().unwrap(),
                                "BTC"
                            ).execute(&pool).await?;
                        }
                        "ETH_USDC" => {
                            sqlx::query!(
                                "INSERT INTO eth_prices (time, price, currency_code) VALUES ($1, $2, $3)", 
                                OffsetDateTime::from_unix_timestamp(data.time.timestamp()).unwrap(),
                                data.price.to_string().parse::<f64>().unwrap(),
                                "ETH"
                            ).execute(&pool).await?;
                        }
                        _ => {}
                    }
                }
            }
            None => {}
        }
    }
}

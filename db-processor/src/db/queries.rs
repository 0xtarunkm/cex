use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub async fn insert_trade_price(
    connection_pool: &PgPool,
    price: f64,
    timestamp: String,
) -> Result<()> {
    let timestamp: DateTime<Utc> = timestamp.parse()?;

    sqlx::query("INSERT INTO sol_prices (time, price) VALUES ($1, $2)")
        .bind(timestamp)
        .bind(price)
        .execute(connection_pool)
        .await?;

    Ok(())
}

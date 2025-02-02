use anyhow::Result;
use sqlx::PgPool;

pub async fn init_db() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")?;
    let connection_pool = PgPool::connect(&database_url).await?;
    Ok(connection_pool)
}

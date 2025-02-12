use anyhow::Result;
use bigdecimal::BigDecimal;
use sqlx::PgPool;

pub async fn update_solana_balance(
    pool: &PgPool,
    public_key: &str,
    lamports: BigDecimal
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE users 
        SET balance = $1
        WHERE solana_public_key = $2
        "#,
        lamports,
        public_key
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_user_wallet(
    pool: &PgPool,
    user_id: i32,
    public_key: &str,
    private_key: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO users (
            user_id, 
            solana_public_key, 
            solana_private_key, 
            balance
        )
        VALUES ($1, $2, $3, 0)
        "#,
        user_id,
        public_key,
        private_key
    )
    .execute(pool)
    .await?;

    Ok(())
}
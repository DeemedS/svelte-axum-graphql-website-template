use sqlx::PgPool;

pub async fn init_pool() -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    Ok(pool)
}

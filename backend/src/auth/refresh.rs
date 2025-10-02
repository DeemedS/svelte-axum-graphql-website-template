use crate::auth::jwt::create_jwt;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// A pair of access token and refresh token to return to client
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

/// Generate a new refresh token (upsert) and store hashed in DB
pub async fn create_refresh_token(pool: &PgPool, user_id: i32) -> anyhow::Result<String> {
    let token = Uuid::new_v4().to_string(); // actual token to return to client

    // Hash token before storing in DB
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let hashed_token = format!("{:x}", hasher.finalize());

    let expires_at = Utc::now() + Duration::days(7);

    // Insert or update refresh token for this user
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id)
        DO UPDATE SET token_hash = $2, expires_at = $3
        "#,
    )
    .bind(user_id)
    .bind(&hashed_token)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(token)
}

/// Verify refresh token, rotate it, and return new access + refresh tokens
pub async fn refresh_access_token(pool: &PgPool, token: &str) -> Option<TokenPair> {
    // Hash incoming token
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let hashed_token = format!("{:x}", hasher.finalize());

    // Check if token exists and not expired
    let row: (i32,) = sqlx::query_as(
        "SELECT user_id FROM refresh_tokens WHERE token_hash = $1 AND expires_at > NOW()",
    )
    .bind(&hashed_token)
    .fetch_one(pool)
    .await
    .ok()?; // return None if invalid/expired

    let user_id = row.0;

    // Rotate: delete old refresh token
    sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok()?; // return None if DB fails

    // Generate new access token
    let access_token = create_jwt(&user_id.to_string());

    // Generate new refresh token and store in DB
    let refresh_token = create_refresh_token(pool, user_id).await.ok()?; // return None if creation fails

    Some(TokenPair {
        access_token,
        refresh_token,
    })
}

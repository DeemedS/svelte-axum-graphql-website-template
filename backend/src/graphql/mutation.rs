use crate::auth::jwt::create_jwt;
use crate::auth::refresh::{create_refresh_token, refresh_access_token, TokenPair};
use async_graphql::{Context, Object, Result, SimpleObject};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::PgPool;

/// Response object for login/refresh mutation
#[derive(SimpleObject)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Register a new user
    async fn register(&self, ctx: &Context<'_>, email: String, password: String) -> Result<bool> {
        let pool = ctx.data::<PgPool>()?;
        let hashed = hash(password, DEFAULT_COST)?; // securely hash password

        let result = sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
            .bind(&email)
            .bind(&hashed)
            .execute(pool)
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(err) => {
                if let Some(db_err) = err.as_database_error() {
                    if db_err.code().as_deref() == Some("23505") {
                        return Err(async_graphql::Error::new("Email already exists"));
                    }
                }
                Err(async_graphql::Error::new(format!(
                    "Database error: {}",
                    err
                )))
            }
        }
    }

    /// Login an existing user
    async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<LoginResponse> {
        let pool = ctx.data::<PgPool>()?;

        let row: (i32, String) =
            sqlx::query_as("SELECT id, password_hash FROM users WHERE email = $1")
                .bind(&email)
                .fetch_one(pool)
                .await
                .map_err(|_| async_graphql::Error::new("User not found"))?;

        if verify(&password, &row.1)? {
            let access_token = create_jwt(&row.0.to_string());
            let refresh_token = create_refresh_token(pool, row.0).await?;
            Ok(LoginResponse {
                access_token,
                refresh_token,
            })
        } else {
            Err(async_graphql::Error::new("Invalid password"))
        }
    }

    /// Refresh access token using refresh token
    async fn refresh_token(&self, ctx: &Context<'_>, token: String) -> Result<LoginResponse> {
        let pool = ctx.data::<PgPool>()?;

        // Use refresh_access_token to get a TokenPair
        let tokens: TokenPair = refresh_access_token(pool, &token)
            .await
            .ok_or_else(|| async_graphql::Error::new("Invalid or expired refresh token"))?;

        Ok(LoginResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        })
    }
}

use crate::auth::jwt::create_jwt;
use crate::auth::refresh::{create_refresh_token, refresh_access_token};
use crate::utils::validation::{validate_email, validate_password};
use async_graphql::{Context, Object, Result, SimpleObject};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::PgPool;

#[derive(SimpleObject)]
pub struct LoginResponse {
    success: bool,
    message: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

#[derive(SimpleObject)]
struct RegisterResponse {
    success: bool,
    message: Option<String>,
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<RegisterResponse> {
        // --- Validation ---
        if let Err(msg) = validate_email(&email) {
            return Ok(RegisterResponse {
                success: false,
                message: Some(msg),
            });
        }

        if let Err(msg) = validate_password(&password) {
            return Ok(RegisterResponse {
                success: false,
                message: Some(msg),
            });
        }

        let hashed = hash(password, DEFAULT_COST)
            .map_err(|_| async_graphql::Error::new("Failed to hash password"))?;

        let pool = ctx.data::<PgPool>()?;
        let result = sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
            .bind(&email)
            .bind(&hashed)
            .execute(pool)
            .await;

        match result {
            Ok(_) => Ok(RegisterResponse {
                success: true,
                message: Some("Registration successful".into()),
            }),
            Err(err) => {
                if let Some(db_err) = err.as_database_error() {
                    if db_err.code().as_deref() == Some("23505") {
                        return Ok(RegisterResponse {
                            success: false,
                            message: Some("Email already exists".into()),
                        });
                    }
                }
                Ok(RegisterResponse {
                    success: false,
                    message: Some("Failed to register user".into()),
                })
            }
        }
    }

    async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<LoginResponse> {
        let pool = ctx.data::<PgPool>()?;

        let row = sqlx::query_as::<_, (i32, String)>(
            "SELECT id, password_hash FROM users WHERE email = $1",
        )
        .bind(&email)
        .fetch_optional(pool)
        .await
        .map_err(|_| async_graphql::Error::new("Database error"))?;

        if let Some((id, hash)) = row {
            if verify(&password, &hash)? {
                let access_token = create_jwt(&id.to_string());
                let refresh_token = create_refresh_token(pool, id).await?;

                return Ok(LoginResponse {
                    success: true,
                    message: Some("Login successful".into()),
                    access_token: Some(access_token),
                    refresh_token: Some(refresh_token),
                });
            } else {
                return Ok(LoginResponse {
                    success: false,
                    message: Some("Invalid username or password".into()),
                    access_token: None,
                    refresh_token: None,
                });
            }
        }

        Ok(LoginResponse {
            success: false,
            message: Some("User not found".into()),
            access_token: None,
            refresh_token: None,
        })
    }

    async fn refresh_token(&self, ctx: &Context<'_>, token: String) -> Result<LoginResponse> {
        let pool = ctx.data::<PgPool>()?;

        // Try to refresh the token
        if let Some(tokens) = refresh_access_token(pool, &token).await {
            Ok(LoginResponse {
                success: true,
                message: Some("Token refreshed successfully".to_string()),
                access_token: Some(tokens.access_token),
                refresh_token: Some(tokens.refresh_token),
            })
        } else {
            Ok(LoginResponse {
                success: false,
                message: Some("Invalid or expired refresh token".to_string()),
                access_token: None,
                refresh_token: None,
            })
        }
    }
}

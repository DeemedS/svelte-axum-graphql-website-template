use crate::auth::jwt::create_jwt;
use crate::auth::refresh::{create_refresh_token, refresh_access_token};
use async_graphql::{Context, Object, Result, SimpleObject};
use bcrypt::{hash, verify, DEFAULT_COST};
use regex::Regex;
use sqlx::PgPool;

/// Response object for login/refresh mutation
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
    /// Register a new user
    async fn register(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<RegisterResponse> {
        // --- Input Validations ---
        if email.trim().is_empty() {
            return Ok(RegisterResponse {
                success: false,
                message: Some("Email is required".to_string()),
            });
        }

        let email_regex = Regex::new(r"^\S+@\S+\.\S+$").unwrap();
        if !email_regex.is_match(&email) {
            return Ok(RegisterResponse {
                success: false,
                message: Some("Invalid email address".to_string()),
            });
        }

        if password.trim().is_empty() {
            return Ok(RegisterResponse {
                success: false,
                message: Some("Password is required".to_string()),
            });
        }

        if password.len() < 8 {
            return Ok(RegisterResponse {
                success: false,
                message: Some("Password must be at least 8 characters".to_string()),
            });
        }

        if !password.chars().any(|c| c.is_ascii_uppercase()) {
            return Ok(RegisterResponse {
                success: false,
                message: Some("Password must contain at least one uppercase letter".to_string()),
            });
        }

        if !password
            .chars()
            .any(|c| "!@#$%^&*(),.?\":{}|<>".contains(c))
        {
            return Ok(RegisterResponse {
                success: false,
                message: Some("Password must contain at least one special character".to_string()),
            });
        }

        // --- Hash password ---
        let hashed = match hash(password, DEFAULT_COST) {
            Ok(h) => h,
            Err(_) => {
                return Ok(RegisterResponse {
                    success: false,
                    message: Some("Failed to hash password".to_string()),
                });
            }
        };

        // --- Insert into database ---
        let pool = ctx.data::<PgPool>()?;
        let result = sqlx::query("INSERT INTO users (email, password_hash) VALUES ($1, $2)")
            .bind(&email)
            .bind(&hashed)
            .execute(pool)
            .await;

        match result {
            Ok(_) => Ok(RegisterResponse {
                success: true,
                message: Some("Registration successful".to_string()),
            }),
            Err(err) => {
                if let Some(db_err) = err.as_database_error() {
                    if db_err.code().as_deref() == Some("23505") {
                        return Ok(RegisterResponse {
                            success: false,
                            message: Some("Email already exists".to_string()),
                        });
                    }
                }
                Ok(RegisterResponse {
                    success: false,
                    message: Some("Failed to register user".to_string()),
                })
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
                Ok(LoginResponse {
                    success: true,
                    message: Some("Login successful".to_string()),
                    access_token: Some(access_token),
                    refresh_token: Some(refresh_token),
                })
            } else {
                Ok(LoginResponse {
                    success: false,
                    message: Some("Invalid username or password".to_string()),
                    access_token: None,
                    refresh_token: None,
                })
            }
        } else {
            Ok(LoginResponse {
                success: false,
                message: Some("User not found".to_string()),
                access_token: None,
                refresh_token: None,
            })
        }
    }

    /// Refresh access token using refresh token
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

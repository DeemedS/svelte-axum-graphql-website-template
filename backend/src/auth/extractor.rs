use crate::auth::jwt::validate_jwt;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::typed_header::TypedHeader;
use headers::{authorization::Bearer, Authorization};
pub struct AuthUser(pub String);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing token".to_string()))?;

        if let Some(claims) = validate_jwt(bearer.token()) {
            Ok(AuthUser(claims.sub))
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string()))
        }
    }
}

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SECRET: &[u8] = b"supersecretkey"; // TODO: load from .env

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn create_jwt(user_id: &str) -> String {
    let expiration =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(60 * 60);

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration.as_secs() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
    .unwrap()
}

pub fn validate_jwt(token: &str) -> Option<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET),
        &Validation::default(),
    )
    .ok()
    .map(|c| c.claims)
}

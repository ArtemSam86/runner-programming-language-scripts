use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

const JWT_SECRET_ENV: &str = "JWT_SECRET";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // username
    pub exp: usize,
}

pub fn create_token(username: &str) -> Result<String> {
    let secret = env::var(JWT_SECRET_ENV).expect("JWT_SECRET must be set");
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
        .map_err(|e| anyhow!("JWT encode error: {}", e))
}

pub fn validate_token(token: &str) -> Result<Claims> {
    let secret = env::var(JWT_SECRET_ENV).expect("JWT_SECRET must be set");
    let validation = Validation::default();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
        .map(|data| data.claims)
        .map_err(|e| anyhow!("JWT decode error: {}", e))
}
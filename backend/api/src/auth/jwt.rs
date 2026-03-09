use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // user id
    pub email: String,
    pub role: String,
    pub institution_id: String,
    pub jti: String,       // JWT ID for token revocation
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            access_token_expiry_minutes: std::env::var("JWT_ACCESS_EXPIRY_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15),
            refresh_token_expiry_days: std::env::var("JWT_REFRESH_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .unwrap_or(7),
        }
    }
}

pub fn create_access_token(
    user_id: &str,
    email: &str,
    role: &str,
    institution_id: &str,
    config: &JwtConfig,
) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::minutes(config.access_token_expiry_minutes);
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        institution_id: institution_id.to_string(),
        jti: Uuid::new_v4().to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn verify_token(token: &str, config: &JwtConfig) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| anyhow::anyhow!("Token validation failed: {}", e))?;
    Ok(data.claims)
}

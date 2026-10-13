//! Bearer JWT extractor — reads Authorization header and validates the token.
use axum::http::{header::AUTHORIZATION, HeaderMap};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{config::Config, errors::AppError, services::auth_service::Claims};

pub fn extract_bearer(cfg: &Config, headers: &HeaderMap) -> Result<Claims, AppError> {
    let raw = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    let token = raw.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}

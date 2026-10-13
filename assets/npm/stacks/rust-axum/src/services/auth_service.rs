//! Auth business logic — login flow.
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{errors::AppError, repositories::user_repository, SharedState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub email: String,
    pub role: String,
    pub exp: i64,
}

pub async fn login(
    state: &SharedState,
    email: &str,
    password: &str,
) -> Result<(String, i64), AppError> {
    let email = email.trim().to_lowercase();
    let user = user_repository::find_by_email(&state.pool, &email).await?
        .ok_or(AppError::Unauthorized)?;

    let parsed = PasswordHash::new(&user.password)
        .map_err(|_| AppError::Unauthorized)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized)?;

    let exp = Utc::now().timestamp() + state.cfg.jwt_expires_secs;
    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        role: user.role.clone(),
        exp,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.cfg.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok((token, state.cfg.jwt_expires_secs))
}

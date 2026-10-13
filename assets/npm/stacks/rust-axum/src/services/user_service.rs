//! User business logic — find / page / create.
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::user::User,
    repositories::user_repository,
    SharedState,
};

pub async fn find_by_id(state: &SharedState, id: Uuid) -> Result<User, AppError> {
    user_repository::find_by_id(&state.pool, id).await?
        .ok_or(AppError::NotFound)
}

pub async fn page(state: &SharedState, page: i64, limit: i64) -> Result<(Vec<User>, i64), AppError> {
    let offset = (page - 1) * limit;
    let items = user_repository::page(&state.pool, offset, limit).await?;
    let total = user_repository::count(&state.pool).await?;
    Ok((items, total))
}

pub async fn create(
    state: &SharedState,
    email: &str,
    password: &str,
    role: &str,
) -> Result<User, AppError> {
    if !["USER", "ADMIN"].contains(&role) {
        return Err(AppError::Validation(format!("role '{}' is not allowed", role)));
    }
    if password.len() < 8 {
        return Err(AppError::Validation("password must be at least 8 chars".into()));
    }

    let email = email.trim().to_lowercase();
    if user_repository::find_by_email(&state.pool, &email).await?.is_some() {
        return Err(AppError::Conflict("email already in use".into()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?
        .to_string();

    let user = user_repository::create(&state.pool, &email, &hash, role).await?;
    Ok(user)
}

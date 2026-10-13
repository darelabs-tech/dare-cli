//! User repository — the only place sqlx is called for the User aggregate.
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, password, role, created_at, updated_at \
         FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, password, role, created_at, updated_at \
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn page(pool: &PgPool, offset: i64, limit: i64) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, password, role, created_at, updated_at \
         FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn create(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    role: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, password, role) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, email, password, role, created_at, updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .fetch_one(pool)
    .await
}

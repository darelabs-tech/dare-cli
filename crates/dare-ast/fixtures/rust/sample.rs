use axum::{routing::get, Router};

pub struct User {
    pub id: String,
}

pub enum Role {
    Admin,
    Member,
}

pub fn app() -> Router {
    Router::new()
        .route("/users", get(|| async { "ok" }))
        .route("/health", get(|| async { "ok" }))
}

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/healthz", get(healthz));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

async fn healthz() -> &'static str {
    "{\"status\":\"ok\"}"
}

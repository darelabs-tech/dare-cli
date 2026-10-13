//! WebSocket echo handler using axum::extract::ws.
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

pub async fn handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(echo)
}

async fn echo(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(t) => {
                let _ = socket.send(Message::Text(t)).await;
            }
            Message::Binary(b) => {
                let _ = socket.send(Message::Binary(b)).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

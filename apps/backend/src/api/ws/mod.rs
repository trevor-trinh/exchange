mod client;
mod server;
mod state;

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::api::ServerMessage;
use state::SocketState;

// Configuration constants
pub(crate) const PING_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
pub(crate) const PONG_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
pub(crate) const UNSUBSCRIBED_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// Create the WebSocket router
pub fn create_ws() -> Router<crate::AppState> {
    Router::new().route("/ws", get(ws_handler))
}

/// WebSocket upgrade handler
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<crate::AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: crate::AppState) {
    // sender sends to client, receiver receives from client
    let (sender, receiver) = socket.split();
    let event_rx = state.event_tx.subscribe();

    // Shared socket state
    let socket_state = Arc::new(RwLock::new(SocketState::new()));

    // Channel for sending acknowledgments from client handler to server sender
    let (ack_tx, ack_rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();

    // Task 1: Handle incoming messages from client (receiver)
    let recv_task = {
        let socket_state = socket_state.clone();
        tokio::spawn(
            async move { client::handle_client_messages(receiver, socket_state, ack_tx).await },
        )
    };

    // Task 2: Send outgoing messages to client (sender)
    let send_task = {
        let socket_state = socket_state.clone();
        tokio::spawn(async move {
            server::handle_server_messages(sender, event_rx, socket_state, ack_rx).await
        })
    };

    // Wait for either task to complete (disconnection)
    tokio::select! {
        _ = recv_task => {
            log::info!("WebSocket receive task ended");
        },
        _ = send_task => {
            log::info!("WebSocket send task ended");
        },
    }

    log::info!("WebSocket connection closed");
}

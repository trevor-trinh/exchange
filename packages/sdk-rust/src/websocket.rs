use crate::error::{SdkError, SdkResult};
use backend::models::api::{ClientMessage, SubscriptionChannel};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// WebSocket client for real-time data streams
pub struct WebSocketClient {
    url: String,
    ping_interval: Duration,
    pong_timeout: Duration,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ping_interval: Duration::from_secs(30),
            pong_timeout: Duration::from_secs(60),
        }
    }

    /// Create a new WebSocket client with custom ping/pong settings
    pub fn with_ping_config(
        url: impl Into<String>,
        ping_interval: Duration,
        pong_timeout: Duration,
    ) -> Self {
        Self {
            url: url.into(),
            ping_interval,
            pong_timeout,
        }
    }

    /// Connect to the WebSocket server and return a handle for communication
    pub async fn connect(&self) -> SdkResult<WebSocketHandle> {
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| SdkError::WebSocketError(e.to_string()))?;

        let (write, read) = ws_stream.split();

        // Create channels for sending/receiving messages
        let (tx_to_ws, mut rx_from_user) = mpsc::unbounded_channel::<ClientMessage>();
        let (tx_to_user, rx_from_ws) = mpsc::unbounded_channel::<serde_json::Value>();
        let (tx_pong_notify, mut rx_pong_notify) = mpsc::unbounded_channel::<()>();

        // Clone ping settings for tasks
        let ping_interval = self.ping_interval;
        let pong_timeout = self.pong_timeout;

        // Spawn task to handle automatic ping/pong
        let ping_tx = tx_to_ws.clone();
        tokio::spawn(async move {
            let mut ping_timer = interval(ping_interval);
            let mut last_pong = Instant::now();

            loop {
                tokio::select! {
                    _ = ping_timer.tick() => {
                        // Check if we've received a pong recently
                        if last_pong.elapsed() > pong_timeout {
                            eprintln!("[WebSocket] No pong received, connection dead");
                            break;
                        }

                        // Send ping
                        if ping_tx.send(ClientMessage::Ping).is_err() {
                            break;
                        }
                    }
                    _ = rx_pong_notify.recv() => {
                        // Update last pong time
                        last_pong = Instant::now();
                    }
                }
            }
        });

        // Spawn task to handle outgoing messages (user -> websocket)
        tokio::spawn(async move {
            let mut write = write;
            while let Some(msg) = rx_from_user.recv().await {
                let json = serde_json::to_string(&msg).unwrap();
                if let Err(e) = write.send(Message::Text(json.into())).await {
                    eprintln!("WebSocket send error: {}", e);
                    break;
                }
            }
        });

        // Spawn task to handle incoming messages (websocket -> user)
        tokio::spawn(async move {
            let mut read = read;
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse as serde_json::Value first since ServerMessage only has Serialize
                        match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(value) => {
                                // Check if this is a pong message
                                if let Some(msg_type) = value.get("type").and_then(|v| v.as_str()) {
                                    if msg_type == "pong" {
                                        let _ = tx_pong_notify.send(());
                                    }
                                }

                                // Convert to ServerMessage representation
                                if tx_to_user.send(value).is_err() {
                                    break; // Receiver dropped
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse server message: {}", e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(WebSocketHandle {
            tx: tx_to_ws,
            rx: rx_from_ws,
        })
    }
}

/// Handle for sending and receiving WebSocket messages
pub struct WebSocketHandle {
    tx: mpsc::UnboundedSender<ClientMessage>,
    rx: mpsc::UnboundedReceiver<serde_json::Value>,
}

impl WebSocketHandle {
    /// Subscribe to a channel
    pub fn subscribe(
        &self,
        channel: SubscriptionChannel,
        market_id: Option<String>,
        user_address: Option<String>,
    ) -> SdkResult<()> {
        self.tx
            .send(ClientMessage::Subscribe {
                channel,
                market_id,
                user_address,
            })
            .map_err(|e| SdkError::WebSocketError(e.to_string()))
    }

    /// Unsubscribe from a channel
    pub fn unsubscribe(
        &self,
        channel: SubscriptionChannel,
        market_id: Option<String>,
        user_address: Option<String>,
    ) -> SdkResult<()> {
        self.tx
            .send(ClientMessage::Unsubscribe {
                channel,
                market_id,
                user_address,
            })
            .map_err(|e| SdkError::WebSocketError(e.to_string()))
    }

    /// Send a ping
    pub fn ping(&self) -> SdkResult<()> {
        self.tx
            .send(ClientMessage::Ping)
            .map_err(|e| SdkError::WebSocketError(e.to_string()))
    }

    /// Receive the next message from the server
    pub async fn recv(&mut self) -> Option<serde_json::Value> {
        self.rx.recv().await
    }

    /// Try to receive a message without blocking
    pub fn try_recv(&mut self) -> Option<serde_json::Value> {
        self.rx.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_construction() {
        let client = WebSocketClient::new("ws://localhost:8001/ws");
        assert_eq!(client.url, "ws://localhost:8001/ws");
    }
}

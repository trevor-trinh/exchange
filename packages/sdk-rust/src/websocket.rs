use crate::error::{SdkError, SdkResult};
use backend::models::api::{ClientMessage, SubscriptionChannel};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// WebSocket client for real-time data streams
pub struct WebSocketClient {
    url: String,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
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

use super::types::{HlMessage, L2BookData, SubscriptionRequest, Subscription, TradeData};
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

const HYPERLIQUID_WS: &str = "wss://api.hyperliquid.xyz/ws";

pub struct HyperliquidClient {
    coin: String,
}

impl HyperliquidClient {
    pub fn new(coin: String) -> Self {
        Self { coin }
    }

    /// Start streaming orderbook and trades
    pub async fn start(
        &self,
    ) -> Result<(mpsc::Receiver<HlMessage>, tokio::task::JoinHandle<()>)> {
        let (tx, rx) = mpsc::channel(1000);

        let coin = self.coin.clone();

        // Spawn WebSocket handler
        let handle = tokio::spawn(async move {
            if let Err(e) = Self::stream(coin, tx).await {
                error!("Hyperliquid stream error: {}", e);
            }
        });

        Ok((rx, handle))
    }

    /// Stream orderbook and trades from Hyperliquid
    async fn stream(coin: String, tx: mpsc::Sender<HlMessage>) -> Result<()> {
        info!("Connecting to Hyperliquid WebSocket for {}", coin);

        let (ws_stream, _) = connect_async(HYPERLIQUID_WS).await?;
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to L2 orderbook
        let l2_sub = SubscriptionRequest {
            method: "subscribe".to_string(),
            subscription: Subscription {
                sub_type: "l2Book".to_string(),
                coin: coin.clone(),
            },
        };

        let l2_msg = serde_json::to_string(&l2_sub)?;
        write.send(Message::Text(l2_msg.into())).await?;
        info!("Subscribed to L2 book for {}", coin);

        // Subscribe to trades
        let trade_sub = SubscriptionRequest {
            method: "subscribe".to_string(),
            subscription: Subscription {
                sub_type: "trades".to_string(),
                coin: coin.clone(),
            },
        };

        let trade_msg = serde_json::to_string(&trade_sub)?;
        write.send(Message::Text(trade_msg.into())).await?;
        info!("Subscribed to trades for {}", coin);

        // Process messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Try to parse as generic message wrapper
                    if let Ok(wrapper) = serde_json::from_str::<serde_json::Value>(&text) {
                        let channel = wrapper.get("channel").and_then(|v| v.as_str());

                        match channel {
                            Some("l2Book") => {
                                if let Ok(book) = serde_json::from_value::<L2BookData>(
                                    wrapper.get("data").cloned().unwrap_or_default(),
                                ) {
                                    if tx.send(HlMessage::L2Book(book)).await.is_err() {
                                        warn!("Receiver dropped");
                                        break;
                                    }
                                }
                            }
                            Some("trades") => {
                                if let Ok(trades) = serde_json::from_value::<Vec<TradeData>>(
                                    wrapper.get("data").cloned().unwrap_or_default(),
                                ) {
                                    if tx.send(HlMessage::Trade(trades)).await.is_err() {
                                        warn!("Receiver dropped");
                                        break;
                                    }
                                }
                            }
                            _ => {
                                debug!("Unknown channel or message: {}", text);
                            }
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    let _ = write.send(Message::Pong(data)).await;
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket closed by server");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

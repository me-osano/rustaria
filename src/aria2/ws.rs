//! WebSocket client for aria2 real-time events.

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::events::Aria2Event;

/// WebSocket client for receiving aria2 events.
#[derive(Clone)]
pub struct WebSocketClient {
    tx: broadcast::Sender<Aria2Event>,
}

#[derive(Debug, Deserialize)]
struct WsNotification {
    method: String,
    params: Vec<WsEventParams>,
}

#[derive(Debug, Deserialize)]
struct WsEventParams {
    gid: String,
}

impl WebSocketClient {
    /// Connect to aria2 WebSocket endpoint.
    pub async fn connect(url: &str) -> Result<Self> {
        let (tx, _) = broadcast::channel(256);
        let client = Self { tx: tx.clone() };

        let url = url.to_string();
        tokio::spawn(async move {
            if let Err(e) = Self::run_connection(&url, tx).await {
                tracing::error!("WebSocket connection error: {}", e);
            }
        });

        Ok(client)
    }

    async fn run_connection(url: &str, tx: broadcast::Sender<Aria2Event>) -> Result<()> {
        let (ws_stream, _) = connect_async(url)
            .await
            .context("Failed to connect to aria2 WebSocket")?;

        let (mut write, mut read) = ws_stream.split();

        tracing::info!("Connected to aria2 WebSocket");

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(notification) = serde_json::from_str::<WsNotification>(&text) {
                        if let Some(params) = notification.params.first() {
                            let event = match notification.method.as_str() {
                                "aria2.onDownloadStart" => {
                                    Aria2Event::DownloadStart(params.gid.clone())
                                }
                                "aria2.onDownloadPause" => {
                                    Aria2Event::DownloadPause(params.gid.clone())
                                }
                                "aria2.onDownloadStop" => {
                                    Aria2Event::DownloadStop(params.gid.clone())
                                }
                                "aria2.onDownloadComplete" => {
                                    Aria2Event::DownloadComplete(params.gid.clone())
                                }
                                "aria2.onDownloadError" => {
                                    Aria2Event::DownloadError(params.gid.clone())
                                }
                                "aria2.onBtDownloadComplete" => {
                                    Aria2Event::BtDownloadComplete(params.gid.clone())
                                }
                                _ => continue,
                            };

                            let _ = tx.send(event);
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    let _ = write.send(Message::Pong(data)).await;
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket closed by server");
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Listen for events and call the handler for each.
    pub async fn listen<F, Fut>(self, handler: F) -> Result<()>
    where
        F: Fn(Aria2Event) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let mut rx = self.tx.subscribe();

        while let Ok(event) = rx.recv().await {
            handler(event).await;
        }

        Ok(())
    }

    /// Subscribe to events.
    pub fn subscribe(&self) -> broadcast::Receiver<Aria2Event> {
        self.tx.subscribe()
    }
}

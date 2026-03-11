//! Native messaging host for browser extension communication.
//!
//! Implements the Chrome/Firefox native messaging protocol.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use super::messaging::Message;

/// Run the native messaging host.
pub async fn run() -> Result<()> {
    tracing::info!("Starting native messaging host");

    loop {
        // Read message length (4 bytes, little-endian)
        let mut len_buf = [0u8; 4];
        if std::io::stdin().read_exact(&mut len_buf).is_err() {
            // EOF or error - extension disconnected
            break;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        if len > 1024 * 1024 {
            // Message too large (> 1MB)
            tracing::error!("Message too large: {} bytes", len);
            continue;
        }

        // Read message body
        let mut msg_buf = vec![0u8; len];
        std::io::stdin().read_exact(&mut msg_buf)?;

        // Parse message
        let message: Message = match serde_json::from_slice(&msg_buf) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to parse message: {}", e);
                send_response(Response::error("Invalid message format"))?;
                continue;
            }
        };

        // Handle message
        let response = handle_message(message).await;
        send_response(response)?;
    }

    Ok(())
}

/// Send a response back to the browser extension.
fn send_response(response: Response) -> Result<()> {
    let json = serde_json::to_vec(&response)?;
    let len = (json.len() as u32).to_le_bytes();

    std::io::stdout().write_all(&len)?;
    std::io::stdout().write_all(&json)?;
    std::io::stdout().flush()?;

    Ok(())
}

/// Handle an incoming message from the browser extension.
async fn handle_message(message: Message) -> Response {
    match message {
        Message::Ping => Response::pong(),

        Message::AddDownload {
            url,
            filename,
            headers,
            cookies,
            referer,
        } => {
            tracing::info!("Received download request: {}", url);

            // TODO: Add to queue via shared state
            // For now, just acknowledge
            Response::success(serde_json::json!({
                "status": "queued",
                "url": url,
            }))
        }

        Message::GetStatus => {
            // TODO: Return actual status
            Response::success(serde_json::json!({
                "active": 0,
                "queued": 0,
                "completed": 0,
            }))
        }

        Message::GetConfig => {
            // TODO: Return relevant config
            Response::success(serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
            }))
        }
    }
}

/// Response sent back to the browser extension.
#[derive(Debug, Serialize)]
pub struct Response {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }

    pub fn pong() -> Self {
        Self {
            success: true,
            data: Some(serde_json::json!({ "pong": true })),
            error: None,
        }
    }
}

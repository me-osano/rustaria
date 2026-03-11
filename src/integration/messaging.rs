//! Message types for browser extension communication.

use serde::{Deserialize, Serialize};

/// Messages received from the browser extension.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Ping to check if host is alive.
    Ping,

    /// Request to add a download.
    AddDownload {
        url: String,
        #[serde(default)]
        filename: Option<String>,
        #[serde(default)]
        headers: Vec<Header>,
        #[serde(default)]
        cookies: Option<String>,
        #[serde(default)]
        referer: Option<String>,
    },

    /// Request current download status.
    GetStatus,

    /// Request configuration.
    GetConfig,
}

/// HTTP header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    /// Format as HTTP header string.
    pub fn to_header_string(&self) -> String {
        format!("{}: {}", self.name, self.value)
    }
}

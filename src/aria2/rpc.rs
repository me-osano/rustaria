//! JSON-RPC 2.0 client for aria2
//!
//! Implements the aria2 JSON-RPC interface methods.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

use super::types::{AddUriOptions, DownloadStatus};

/// JSON-RPC 2.0 client for aria2.
pub struct RpcClient {
    client: Client,
    url: String,
    secret: Option<String>,
    request_id: AtomicU64,
}

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl RpcClient {
    /// Create a new RPC client.
    pub fn new(url: &str, secret: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let secret = if secret.is_empty() {
            None
        } else {
            Some(format!("token:{}", secret))
        };

        Ok(Self {
            client,
            url: url.to_string(),
            secret,
            request_id: AtomicU64::new(1),
        })
    }

    /// Get the WebSocket URL derived from the RPC URL.
    pub fn ws_url(&self) -> String {
        self.url
            .replace("http://", "ws://")
            .replace("https://", "wss://")
            .replace("/jsonrpc", "/jsonrpc")
    }

    /// Execute an RPC call.
    async fn call(&self, method: &str, params: Vec<Value>) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        // Prepend secret token if configured
        let params = if let Some(ref secret) = self.secret {
            let mut p = vec![json!(secret)];
            p.extend(params);
            p
        } else {
            params
        };

        let request = RpcRequest {
            jsonrpc: "2.0",
            id,
            method: format!("aria2.{}", method),
            params,
        };

        let response: RpcResponse = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .context("Failed to send RPC request")?
            .json()
            .await
            .context("Failed to parse RPC response")?;

        if let Some(error) = response.error {
            anyhow::bail!("aria2 RPC error {}: {}", error.code, error.message);
        }

        response.result.context("Empty RPC response")
    }

    /// Add a URI for download.
    pub async fn add_uri(&self, uri: &str, options: AddUriOptions) -> Result<String> {
        let result = self
            .call("addUri", vec![json!([uri]), json!(options)])
            .await?;

        result
            .as_str()
            .map(|s| s.to_string())
            .context("Invalid GID in response")
    }

    /// Pause a download.
    pub async fn pause(&self, gid: &str) -> Result<()> {
        self.call("pause", vec![json!(gid)]).await?;
        Ok(())
    }

    /// Unpause a download.
    pub async fn unpause(&self, gid: &str) -> Result<()> {
        self.call("unpause", vec![json!(gid)]).await?;
        Ok(())
    }

    /// Remove a download.
    pub async fn remove(&self, gid: &str) -> Result<()> {
        self.call("remove", vec![json!(gid)]).await?;
        Ok(())
    }

    /// Force remove a download.
    pub async fn force_remove(&self, gid: &str) -> Result<()> {
        self.call("forceRemove", vec![json!(gid)]).await?;
        Ok(())
    }

    /// Get download status.
    pub async fn tell_status(&self, gid: &str) -> Result<DownloadStatus> {
        let result = self.call("tellStatus", vec![json!(gid)]).await?;
        serde_json::from_value(result).context("Failed to parse download status")
    }

    /// Get global statistics.
    pub async fn get_global_stat(&self) -> Result<Value> {
        self.call("getGlobalStat", vec![]).await
    }

    /// Get active downloads.
    pub async fn tell_active(&self) -> Result<Vec<DownloadStatus>> {
        let result = self.call("tellActive", vec![]).await?;
        serde_json::from_value(result).context("Failed to parse active downloads")
    }

    /// Get waiting downloads.
    pub async fn tell_waiting(&self, offset: i64, count: i64) -> Result<Vec<DownloadStatus>> {
        let result = self
            .call("tellWaiting", vec![json!(offset), json!(count)])
            .await?;
        serde_json::from_value(result).context("Failed to parse waiting downloads")
    }

    /// Get stopped downloads.
    pub async fn tell_stopped(&self, offset: i64, count: i64) -> Result<Vec<DownloadStatus>> {
        let result = self
            .call("tellStopped", vec![json!(offset), json!(count)])
            .await?;
        serde_json::from_value(result).context("Failed to parse stopped downloads")
    }

    /// Shutdown aria2.
    pub async fn shutdown(&self) -> Result<()> {
        self.call("shutdown", vec![]).await?;
        Ok(())
    }
}

//! aria2 bridge layer
//!
//! Handles communication with the aria2c daemon via JSON-RPC and WebSocket,
//! manages the aria2c process lifecycle, and maps internal job IDs to aria2 GIDs.

mod daemon;
mod events;
mod methods;
mod rpc;
mod session;
mod supervisor;
mod types;
mod ws;

pub use daemon::Daemon;
pub use events::{Aria2Event, EventHandler};
pub use rpc::RpcClient;
pub use session::SessionManager;
pub use types::*;
pub use ws::WebSocketClient;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::queue::JobQueue;

/// Main aria2 interface combining RPC, WebSocket, and process management.
#[derive(Clone)]
pub struct Aria2 {
    pub rpc: Arc<RpcClient>,
    pub ws: Arc<RwLock<Option<WebSocketClient>>>,
    pub daemon: Arc<Daemon>,
    pub session: Arc<SessionManager>,
}

impl Aria2 {
    /// Create a new aria2 interface from configuration.
    pub async fn new(config: &Config) -> Result<Self> {
        let daemon = Daemon::new(config)?;

        // Start aria2c if configured
        if config.aria2.auto_start {
            daemon.start().await?;
        }

        let rpc = RpcClient::new(&config.aria2.rpc_url, &config.aria2.rpc_secret)?;
        let session = SessionManager::new(&config.general.data_dir)?;

        Ok(Self {
            rpc: Arc::new(rpc),
            ws: Arc::new(RwLock::new(None)),
            daemon: Arc::new(daemon),
            session: Arc::new(session),
        })
    }

    /// Connect to aria2 WebSocket and listen for real-time events.
    pub async fn listen_events(self, queue: JobQueue) -> Result<()> {
        let ws_url = self.rpc.ws_url();
        let client = WebSocketClient::connect(&ws_url).await?;

        *self.ws.write().await = Some(client.clone());

        client.listen(move |event| {
            let queue = queue.clone();
            let session = self.session.clone();
            async move {
                EventHandler::handle(event, &queue, &session).await
            }
        }).await
    }

    /// Add a download URI and return the GID.
    pub async fn add_uri(&self, uri: &str, options: AddUriOptions) -> Result<String> {
        self.rpc.add_uri(uri, options).await
    }

    /// Pause a download by GID.
    pub async fn pause(&self, gid: &str) -> Result<()> {
        self.rpc.pause(gid).await
    }

    /// Resume a paused download.
    pub async fn resume(&self, gid: &str) -> Result<()> {
        self.rpc.unpause(gid).await
    }

    /// Remove a download.
    pub async fn remove(&self, gid: &str) -> Result<()> {
        self.rpc.remove(gid).await
    }

    /// Get download status.
    pub async fn status(&self, gid: &str) -> Result<DownloadStatus> {
        self.rpc.tell_status(gid).await
    }
}

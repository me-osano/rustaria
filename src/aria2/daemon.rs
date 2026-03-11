//! aria2c process management
//!
//! Spawns and supervises the aria2c daemon process.

use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;

use crate::config::Config;

/// Manages the aria2c daemon process lifecycle.
pub struct Daemon {
    binary_path: String,
    extra_args: Vec<String>,
    rpc_port: u16,
    rpc_secret: String,
    child: RwLock<Option<Child>>,
}

impl Daemon {
    /// Create a new daemon manager from configuration.
    pub fn new(config: &Config) -> Result<Self> {
        let binary_path = if config.aria2.binary_path.is_empty() {
            "aria2c".to_string()
        } else {
            config.aria2.binary_path.clone()
        };

        // Parse port from RPC URL
        let rpc_port = config
            .aria2
            .rpc_url
            .split(':')
            .last()
            .and_then(|s| s.trim_end_matches("/jsonrpc").parse().ok())
            .unwrap_or(6800);

        Ok(Self {
            binary_path,
            extra_args: config.aria2.extra_args.clone(),
            rpc_port,
            rpc_secret: config.aria2.rpc_secret.clone(),
            child: RwLock::new(None),
        })
    }

    /// Start the aria2c daemon.
    pub async fn start(&self) -> Result<()> {
        let mut guard = self.child.write().await;

        if guard.is_some() {
            tracing::warn!("aria2c daemon already running");
            return Ok(());
        }

        let mut cmd = Command::new(&self.binary_path);

        cmd.arg("--enable-rpc")
            .arg(format!("--rpc-listen-port={}", self.rpc_port))
            .arg("--rpc-listen-all=false");

        if !self.rpc_secret.is_empty() {
            cmd.arg(format!("--rpc-secret={}", self.rpc_secret));
        }

        for arg in &self.extra_args {
            cmd.arg(arg);
        }

        cmd.stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let child = cmd.spawn().context("Failed to start aria2c")?;
        tracing::info!("Started aria2c daemon (PID: {:?})", child.id());

        *guard = Some(child);

        // Give aria2c a moment to initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        Ok(())
    }

    /// Stop the aria2c daemon.
    pub async fn stop(&self) -> Result<()> {
        let mut guard = self.child.write().await;

        if let Some(mut child) = guard.take() {
            tracing::info!("Stopping aria2c daemon...");
            child.kill().await.context("Failed to kill aria2c")?;
        }

        Ok(())
    }

    /// Check if the daemon is running.
    pub async fn is_running(&self) -> bool {
        let guard = self.child.read().await;
        guard.is_some()
    }

    /// Restart the daemon.
    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
        self.start().await
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        // Child processes are killed on drop due to kill_on_drop(true)
    }
}

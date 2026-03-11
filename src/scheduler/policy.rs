//! Scheduling policy definitions.

use anyhow::Result;
use crate::config::Config;

/// Scheduling policy configuration.
#[derive(Debug, Clone)]
pub struct Policy {
    /// Whether scheduling is enabled.
    pub enabled: bool,

    /// Cron expression for active download window.
    pub active_window: Option<String>,

    /// Bandwidth limit in bytes/sec (0 = unlimited).
    pub bandwidth_limit: u64,

    /// Whether we're currently in an active window.
    in_window: bool,
}

impl Policy {
    /// Create a policy from configuration.
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(Self {
            enabled: config.scheduler.enabled,
            active_window: config.scheduler.active_window.clone(),
            bandwidth_limit: config.scheduler.bandwidth_limit,
            in_window: true, // Default to active
        })
    }

    /// Check if we're in an active download window.
    pub fn is_active_window(&self) -> bool {
        if !self.enabled {
            return true; // Always active if scheduling disabled
        }

        if self.active_window.is_none() {
            return true; // Always active if no window defined
        }

        self.in_window
    }

    /// Update the active window state based on current time.
    pub fn update_window(&mut self) {
        if let Some(ref _cron) = self.active_window {
            // TODO: Parse cron expression and check against current time
            // For now, always active
            self.in_window = true;
        }
    }

    /// Get the current bandwidth limit.
    pub fn get_bandwidth_limit(&self) -> u64 {
        self.bandwidth_limit
    }

    /// Set a temporary bandwidth limit.
    pub fn set_bandwidth_limit(&mut self, limit: u64) {
        self.bandwidth_limit = limit;
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            enabled: false,
            active_window: None,
            bandwidth_limit: 0,
            in_window: true,
        }
    }
}

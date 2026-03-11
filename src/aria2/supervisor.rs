//! aria2c process supervisor
//!
//! Manages the lifecycle of the aria2c daemon process, including:
//! - Health monitoring and automatic restart on failure
//! - Graceful shutdown handling
//! - Restart policy configuration

use std::time::Duration;
use tokio::process::Child;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Restart policy for the aria2c daemon
#[derive(Debug, Clone)]
pub enum RestartPolicy {
    /// Never restart on failure
    Never,
    /// Always restart on failure
    Always,
    /// Restart up to N times with exponential backoff
    OnFailure {
        max_retries: u32,
        base_delay: Duration,
    },
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self::OnFailure {
            max_retries: 5,
            base_delay: Duration::from_secs(1),
        }
    }
}

/// Supervisor state for tracking restart attempts
#[derive(Debug)]
pub struct SupervisorState {
    restart_count: u32,
    policy: RestartPolicy,
    is_running: bool,
}

impl SupervisorState {
    pub fn new(policy: RestartPolicy) -> Self {
        Self {
            restart_count: 0,
            policy,
            is_running: false,
        }
    }

    /// Check if we should attempt a restart based on the policy
    pub fn should_restart(&self) -> bool {
        match &self.policy {
            RestartPolicy::Never => false,
            RestartPolicy::Always => true,
            RestartPolicy::OnFailure { max_retries, .. } => self.restart_count < *max_retries,
        }
    }

    /// Get the delay before the next restart attempt
    pub fn restart_delay(&self) -> Duration {
        match &self.policy {
            RestartPolicy::OnFailure { base_delay, .. } => {
                // Exponential backoff: base_delay * 2^restart_count
                *base_delay * 2u32.saturating_pow(self.restart_count)
            }
            _ => Duration::from_secs(1),
        }
    }

    /// Record a restart attempt
    pub fn record_restart(&mut self) {
        self.restart_count += 1;
        info!(
            restart_count = self.restart_count,
            "Recording restart attempt"
        );
    }

    /// Reset restart counter (called on successful startup)
    pub fn reset(&mut self) {
        self.restart_count = 0;
        self.is_running = true;
    }

    /// Mark as stopped
    pub fn mark_stopped(&mut self) {
        self.is_running = false;
    }
}

/// Aria2 process supervisor
pub struct Aria2Supervisor {
    state: SupervisorState,
}

impl Aria2Supervisor {
    pub fn new(policy: RestartPolicy) -> Self {
        Self {
            state: SupervisorState::new(policy),
        }
    }

    /// Supervise a running aria2c process
    /// Returns when the process exits and should not be restarted
    pub async fn supervise<F, Fut>(&mut self, mut spawn_fn: F) -> anyhow::Result<()>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Child>>,
    {
        loop {
            info!("Starting aria2c process");
            
            match spawn_fn().await {
                Ok(mut child) => {
                    self.state.reset();
                    
                    // Wait for the process to exit
                    match child.wait().await {
                        Ok(status) => {
                            self.state.mark_stopped();
                            
                            if status.success() {
                                info!("aria2c exited successfully");
                                return Ok(());
                            } else {
                                warn!(exit_code = ?status.code(), "aria2c exited with error");
                            }
                        }
                        Err(e) => {
                            self.state.mark_stopped();
                            error!(error = %e, "Failed to wait for aria2c process");
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to spawn aria2c process");
                }
            }

            // Check if we should restart
            if !self.state.should_restart() {
                error!("Restart policy exhausted, giving up");
                anyhow::bail!("aria2c process failed and restart policy exhausted");
            }

            let delay = self.state.restart_delay();
            warn!(delay_secs = delay.as_secs(), "Restarting aria2c after delay");
            sleep(delay).await;
            
            self.state.record_restart();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restart_policy_default() {
        let state = SupervisorState::new(RestartPolicy::default());
        assert!(state.should_restart());
    }

    #[test]
    fn test_restart_policy_never() {
        let state = SupervisorState::new(RestartPolicy::Never);
        assert!(!state.should_restart());
    }

    #[test]
    fn test_exponential_backoff() {
        let mut state = SupervisorState::new(RestartPolicy::OnFailure {
            max_retries: 5,
            base_delay: Duration::from_secs(1),
        });

        assert_eq!(state.restart_delay(), Duration::from_secs(1));
        state.record_restart();
        assert_eq!(state.restart_delay(), Duration::from_secs(2));
        state.record_restart();
        assert_eq!(state.restart_delay(), Duration::from_secs(4));
    }
}

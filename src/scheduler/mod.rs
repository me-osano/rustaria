//! Scheduler module
//!
//! Time-based scheduling, concurrency limits, and bandwidth throttling.

mod policy;

pub use policy::Policy;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::queue::JobQueue;

/// Download scheduler.
#[derive(Clone)]
pub struct Scheduler {
    queue: JobQueue,
    policy: Arc<RwLock<Policy>>,
    max_concurrent: usize,
}

impl Scheduler {
    /// Create a new scheduler.
    pub fn new(config: &Config, queue: JobQueue) -> Result<Self> {
        let policy = Policy::from_config(config)?;

        Ok(Self {
            queue,
            policy: Arc::new(RwLock::new(policy)),
            max_concurrent: config.general.max_concurrent,
        })
    }

    /// Run the scheduler loop.
    pub async fn run(self) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            // Check if scheduling is enabled
            let policy = self.policy.read().await;
            if !policy.is_active_window() {
                continue;
            }
            drop(policy);

            // Check concurrency limit
            let active_count = self.queue.active_count().await?;
            if active_count >= self.max_concurrent {
                continue;
            }

            // Start next queued job
            let slots_available = self.max_concurrent - active_count;
            for _ in 0..slots_available {
                if let Some(job) = self.queue.next_queued().await? {
                    if let Err(e) = self.queue.start(job.id).await {
                        tracing::error!("Failed to start job {}: {}", job.id, e);
                    }
                } else {
                    break;
                }
            }
        }
    }

    /// Update scheduler policy.
    pub async fn update_policy(&self, policy: Policy) {
        *self.policy.write().await = policy;
    }

    /// Set maximum concurrent downloads.
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max;
    }
}

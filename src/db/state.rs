//! State persistence for queue and scheduler
//!
//! Persists application state across restarts using TOML serialization.
//! This includes queue state, scheduler state, and resume data.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

/// Path to the state file
const STATE_FILE_NAME: &str = "rustaria-state.toml";

/// Persistent application state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    /// Queue state
    pub queue: QueueState,
    /// Scheduler state
    pub scheduler: SchedulerState,
    /// Job resume data (job_id -> resume info)
    pub resume_data: HashMap<String, ResumeData>,
    /// Last saved timestamp
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub last_saved: Option<chrono::DateTime<chrono::Utc>>,
}

/// Persisted queue state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueState {
    /// List of job IDs in queue order
    pub job_order: Vec<String>,
    /// Jobs that were active when saved (should resume on restart)
    pub active_jobs: Vec<String>,
    /// Jobs that were paused
    pub paused_jobs: Vec<String>,
}

/// Persisted scheduler state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerState {
    /// Whether the scheduler was running
    pub is_running: bool,
    /// Current concurrency limit
    pub concurrency_limit: Option<u32>,
    /// Current bandwidth limit in bytes/sec
    pub bandwidth_limit: Option<u64>,
    /// Active schedule name (if any)
    pub active_schedule: Option<String>,
}

/// Resume data for a single job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeData {
    /// Job ID
    pub job_id: String,
    /// aria2 GID (if available)
    pub gid: Option<String>,
    /// Original URL
    pub url: String,
    /// Download directory
    pub dir: Option<String>,
    /// Output filename
    pub filename: Option<String>,
    /// Bytes downloaded
    pub downloaded_bytes: u64,
    /// Total file size (if known)
    pub total_bytes: Option<u64>,
    /// HTTP headers to restore
    pub headers: HashMap<String, String>,
    /// Cookies to restore
    pub cookies: Option<String>,
}

/// State store manager
pub struct StateStore {
    state_path: PathBuf,
    state: AppState,
}

impl StateStore {
    /// Create a new state store
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let state_path = data_dir.as_ref().join(STATE_FILE_NAME);
        Self {
            state_path,
            state: AppState::default(),
        }
    }

    /// Load state from disk
    pub async fn load(&mut self) -> Result<()> {
        if !self.state_path.exists() {
            debug!(path = %self.state_path.display(), "No existing state file found");
            return Ok(());
        }

        let contents = fs::read_to_string(&self.state_path)
            .await
            .context("Failed to read state file")?;

        self.state = toml::from_str(&contents).context("Failed to parse state file")?;

        info!(
            path = %self.state_path.display(),
            jobs = self.state.queue.job_order.len(),
            "Loaded state from disk"
        );

        Ok(())
    }

    /// Save state to disk
    pub async fn save(&mut self) -> Result<()> {
        self.state.last_saved = Some(chrono::Utc::now());

        let contents = toml::to_string_pretty(&self.state).context("Failed to serialize state")?;

        // Ensure parent directory exists
        if let Some(parent) = self.state_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("Failed to create state directory")?;
        }

        // Write atomically using a temp file
        let temp_path = self.state_path.with_extension("tmp");
        fs::write(&temp_path, &contents)
            .await
            .context("Failed to write temp state file")?;
        fs::rename(&temp_path, &self.state_path)
            .await
            .context("Failed to rename state file")?;

        debug!(path = %self.state_path.display(), "Saved state to disk");
        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    /// Update queue state
    pub fn update_queue(&mut self, queue_state: QueueState) {
        self.state.queue = queue_state;
    }

    /// Update scheduler state
    pub fn update_scheduler(&mut self, scheduler_state: SchedulerState) {
        self.state.scheduler = scheduler_state;
    }

    /// Add or update resume data for a job
    pub fn set_resume_data(&mut self, job_id: String, data: ResumeData) {
        self.state.resume_data.insert(job_id, data);
    }

    /// Remove resume data for a job (e.g., when completed)
    pub fn remove_resume_data(&mut self, job_id: &str) -> Option<ResumeData> {
        self.state.resume_data.remove(job_id)
    }

    /// Get resume data for a job
    pub fn get_resume_data(&self, job_id: &str) -> Option<&ResumeData> {
        self.state.resume_data.get(job_id)
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.state = AppState::default();
    }

    /// Delete state file from disk
    pub async fn delete(&self) -> Result<()> {
        if self.state_path.exists() {
            fs::remove_file(&self.state_path)
                .await
                .context("Failed to delete state file")?;
            info!(path = %self.state_path.display(), "Deleted state file");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_state_save_load() {
        let dir = tempdir().unwrap();
        let mut store = StateStore::new(dir.path());

        // Set some state
        store.state_mut().queue.job_order = vec!["job1".to_string(), "job2".to_string()];
        store.state_mut().scheduler.is_running = true;

        // Save and reload
        store.save().await.unwrap();

        let mut store2 = StateStore::new(dir.path());
        store2.load().await.unwrap();

        assert_eq!(store2.state().queue.job_order.len(), 2);
        assert!(store2.state().scheduler.is_running);
    }

    #[tokio::test]
    async fn test_resume_data() {
        let dir = tempdir().unwrap();
        let mut store = StateStore::new(dir.path());

        let resume = ResumeData {
            job_id: "job1".to_string(),
            gid: Some("abc123".to_string()),
            url: "https://example.com/file.zip".to_string(),
            dir: Some("/downloads".to_string()),
            filename: Some("file.zip".to_string()),
            downloaded_bytes: 1024,
            total_bytes: Some(4096),
            headers: HashMap::new(),
            cookies: None,
        };

        store.set_resume_data("job1".to_string(), resume);
        assert!(store.get_resume_data("job1").is_some());

        store.remove_resume_data("job1");
        assert!(store.get_resume_data("job1").is_none());
    }
}

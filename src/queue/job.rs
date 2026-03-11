//! Job struct and status definitions.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// A download job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Internal job ID.
    pub id: i64,

    /// Download URL.
    pub url: String,

    /// Current status.
    pub status: JobStatus,

    /// aria2 GID (if submitted to aria2).
    pub gid: Option<String>,

    /// Output filename.
    pub filename: Option<String>,

    /// Download directory.
    pub dir: Option<String>,

    /// Full output path after completion.
    pub output_path: Option<String>,

    /// Total file size in bytes.
    pub total_size: u64,

    /// Downloaded bytes.
    pub downloaded: u64,

    /// Download progress (0-100).
    pub progress: f64,

    /// Current download speed in bytes/sec.
    pub speed: u64,

    /// Category for organization.
    pub category: Option<String>,

    /// Tags for filtering.
    pub tags: Vec<String>,

    /// HTTP headers to send.
    pub headers: Vec<String>,

    /// Referer URL.
    pub referer: Option<String>,

    /// User agent string.
    pub user_agent: Option<String>,

    /// Error message if failed.
    pub error: Option<String>,

    /// Creation timestamp.
    pub created_at: i64,

    /// Last update timestamp.
    pub updated_at: i64,

    /// Completion timestamp.
    pub completed_at: Option<i64>,
}

impl Job {
    /// Create a new job.
    pub fn new(
        url: String,
        filename: Option<String>,
        dir: Option<PathBuf>,
        category: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();

        Self {
            id: 0, // Will be set by database
            url,
            status: JobStatus::Queued,
            gid: None,
            filename,
            dir: dir.map(|p| p.to_string_lossy().to_string()),
            output_path: None,
            total_size: 0,
            downloaded: 0,
            progress: 0.0,
            speed: 0,
            category,
            tags,
            headers: vec![],
            referer: None,
            user_agent: None,
            error: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// Check if job is in a final state.
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Complete | JobStatus::Failed | JobStatus::Stopped
        )
    }

    /// Check if job can be started.
    pub fn can_start(&self) -> bool {
        matches!(self.status, JobStatus::Queued | JobStatus::Paused)
    }

    /// Check if job can be paused.
    pub fn can_pause(&self) -> bool {
        matches!(self.status, JobStatus::Active)
    }
}

/// Job status enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job created, waiting in queue.
    Queued,

    /// Job is actively downloading.
    Active,

    /// Job is paused.
    Paused,

    /// Job was stopped by user.
    Stopped,

    /// Job completed successfully.
    Complete,

    /// Job failed with an error.
    Failed,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Active => write!(f, "active"),
            Self::Paused => write!(f, "paused"),
            Self::Stopped => write!(f, "stopped"),
            Self::Complete => write!(f, "complete"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "active" => Ok(Self::Active),
            "paused" => Ok(Self::Paused),
            "stopped" => Ok(Self::Stopped),
            "complete" => Ok(Self::Complete),
            "failed" => Ok(Self::Failed),
            _ => anyhow::bail!("Unknown job status: {}", s),
        }
    }
}

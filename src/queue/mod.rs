//! Job queue management
//!
//! Manages download job lifecycle: Created → Queued → Active → Paused → Done → Failed

mod job;
mod state_machine;

pub use job::{Job, JobStatus};
pub use state_machine::StateMachine;

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use crate::aria2::Aria2;
use crate::db::Database;

/// Manages the download job queue.
#[derive(Clone)]
pub struct JobQueue {
    db: Arc<Database>,
    aria2: Aria2,
}

impl JobQueue {
    /// Create a new job queue.
    pub fn new(db: Arc<Database>, aria2: Aria2) -> Self {
        Self { db, aria2 }
    }

    /// Add a new download job.
    pub async fn add(
        &self,
        url: &str,
        filename: Option<String>,
        dir: Option<PathBuf>,
        category: Option<String>,
        tags: Vec<String>,
    ) -> Result<i64> {
        let job = Job::new(url.to_string(), filename, dir, category, tags);
        let job_id = self.db.insert_job(&job).await?;

        tracing::info!("Created job {}: {}", job_id, url);
        Ok(job_id)
    }

    /// Start a queued job.
    pub async fn start(&self, job_id: i64) -> Result<()> {
        let job = self.db.get_job(job_id).await?;

        if !StateMachine::can_transition(&job.status, &JobStatus::Active) {
            anyhow::bail!("Cannot start job in {} state", job.status);
        }

        // Add to aria2
        let options = crate::aria2::AddUriOptions {
            dir: job.dir.clone(),
            out: job.filename.clone(),
            ..Default::default()
        };

        let gid = self.aria2.add_uri(&job.url, options).await?;

        // Update job with GID
        self.db.update_job_gid(job_id, &gid).await?;
        self.db.update_job_status(job_id, JobStatus::Active).await?;

        // Register GID mapping
        self.aria2.session.register(gid, job_id).await;

        tracing::info!("Started job {}", job_id);
        Ok(())
    }

    /// Pause a job.
    pub async fn pause(&self, job_id: i64) -> Result<()> {
        let job = self.db.get_job(job_id).await?;

        if let Some(ref gid) = job.gid {
            self.aria2.pause(gid).await?;
        }

        self.db.update_job_status(job_id, JobStatus::Paused).await?;
        tracing::info!("Paused job {}", job_id);
        Ok(())
    }

    /// Resume a paused job.
    pub async fn resume(&self, job_id: i64) -> Result<()> {
        let job = self.db.get_job(job_id).await?;

        if let Some(ref gid) = job.gid {
            self.aria2.resume(gid).await?;
        }

        self.db.update_job_status(job_id, JobStatus::Active).await?;
        tracing::info!("Resumed job {}", job_id);
        Ok(())
    }

    /// Remove a job.
    pub async fn remove(&self, job_id: i64, delete_files: bool) -> Result<()> {
        let job = self.db.get_job(job_id).await?;

        // Remove from aria2 if active
        if let Some(ref gid) = job.gid {
            let _ = self.aria2.remove(gid).await;
            self.aria2.session.remove_by_gid(gid).await;
        }

        // Delete files if requested
        if delete_files {
            if let Some(ref path) = job.output_path {
                if std::path::Path::new(path).exists() {
                    std::fs::remove_file(path)?;
                }
            }
        }

        self.db.delete_job(job_id).await?;
        tracing::info!("Removed job {}", job_id);
        Ok(())
    }

    /// Pause all active jobs.
    pub async fn pause_all(&self) -> Result<()> {
        let jobs = self.db.get_jobs_by_status(JobStatus::Active).await?;
        for job in jobs {
            self.pause(job.id).await?;
        }
        Ok(())
    }

    /// Resume all paused jobs.
    pub async fn resume_all(&self) -> Result<()> {
        let jobs = self.db.get_jobs_by_status(JobStatus::Paused).await?;
        for job in jobs {
            self.resume(job.id).await?;
        }
        Ok(())
    }

    /// Get a job by ID.
    pub async fn get(&self, job_id: i64) -> Result<Job> {
        self.db.get_job(job_id).await
    }

    /// List jobs with optional filters.
    pub async fn list(&self, status: Option<&str>, limit: Option<usize>) -> Result<Vec<Job>> {
        self.db.list_jobs(status, limit).await
    }

    /// Set job to active state.
    pub async fn set_active(&self, job_id: i64) -> Result<()> {
        self.db.update_job_status(job_id, JobStatus::Active).await
    }

    /// Set job to paused state.
    pub async fn set_paused(&self, job_id: i64) -> Result<()> {
        self.db.update_job_status(job_id, JobStatus::Paused).await
    }

    /// Set job to stopped state.
    pub async fn set_stopped(&self, job_id: i64) -> Result<()> {
        self.db.update_job_status(job_id, JobStatus::Stopped).await
    }

    /// Set job to complete state.
    pub async fn set_complete(&self, job_id: i64) -> Result<()> {
        self.db.update_job_status(job_id, JobStatus::Complete).await
    }

    /// Set job to failed state.
    pub async fn set_failed(&self, job_id: i64) -> Result<()> {
        self.db.update_job_status(job_id, JobStatus::Failed).await
    }

    /// Get count of active downloads.
    pub async fn active_count(&self) -> Result<usize> {
        let jobs = self.db.get_jobs_by_status(JobStatus::Active).await?;
        Ok(jobs.len())
    }

    /// Get next queued job.
    pub async fn next_queued(&self) -> Result<Option<Job>> {
        self.db.get_next_queued_job().await
    }
}

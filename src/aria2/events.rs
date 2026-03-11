//! aria2 event types and handlers.

use crate::queue::JobQueue;
use super::SessionManager;

/// Events received from aria2 via WebSocket.
#[derive(Debug, Clone)]
pub enum Aria2Event {
    DownloadStart(String),
    DownloadPause(String),
    DownloadStop(String),
    DownloadComplete(String),
    DownloadError(String),
    BtDownloadComplete(String),
}

impl Aria2Event {
    /// Get the GID associated with this event.
    pub fn gid(&self) -> &str {
        match self {
            Self::DownloadStart(gid) => gid,
            Self::DownloadPause(gid) => gid,
            Self::DownloadStop(gid) => gid,
            Self::DownloadComplete(gid) => gid,
            Self::DownloadError(gid) => gid,
            Self::BtDownloadComplete(gid) => gid,
        }
    }
}

/// Handles aria2 events and updates internal state.
pub struct EventHandler;

impl EventHandler {
    /// Process an aria2 event.
    pub async fn handle(event: Aria2Event, queue: &JobQueue, session: &SessionManager) {
        let gid = event.gid();

        // Look up internal job ID from GID
        let job_id = match session.get_job_id(gid).await {
            Some(id) => id,
            None => {
                tracing::warn!("Received event for unknown GID: {}", gid);
                return;
            }
        };

        match event {
            Aria2Event::DownloadStart(_) => {
                tracing::info!("Download started: job_id={}", job_id);
                if let Err(e) = queue.set_active(job_id).await {
                    tracing::error!("Failed to update job state: {}", e);
                }
            }
            Aria2Event::DownloadPause(_) => {
                tracing::info!("Download paused: job_id={}", job_id);
                if let Err(e) = queue.set_paused(job_id).await {
                    tracing::error!("Failed to update job state: {}", e);
                }
            }
            Aria2Event::DownloadStop(_) => {
                tracing::info!("Download stopped: job_id={}", job_id);
                if let Err(e) = queue.set_stopped(job_id).await {
                    tracing::error!("Failed to update job state: {}", e);
                }
            }
            Aria2Event::DownloadComplete(_) | Aria2Event::BtDownloadComplete(_) => {
                tracing::info!("Download complete: job_id={}", job_id);
                if let Err(e) = queue.set_complete(job_id).await {
                    tracing::error!("Failed to update job state: {}", e);
                }

                // Trigger post-processing
                // TODO: spawn post-processing task
            }
            Aria2Event::DownloadError(_) => {
                tracing::error!("Download error: job_id={}", job_id);
                if let Err(e) = queue.set_failed(job_id).await {
                    tracing::error!("Failed to update job state: {}", e);
                }
            }
        }
    }
}

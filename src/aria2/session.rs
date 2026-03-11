//! Session serialization and GID mapping.
//!
//! Persists aria2 session files and maps internal job IDs to aria2 GIDs.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// Manages the mapping between internal job IDs and aria2 GIDs.
pub struct SessionManager {
    data_dir: PathBuf,
    /// Maps GID -> Job ID
    gid_to_job: RwLock<HashMap<String, i64>>,
    /// Maps Job ID -> GID
    job_to_gid: RwLock<HashMap<i64, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionData {
    mappings: Vec<(String, i64)>,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new(data_dir: &str) -> Result<Self> {
        let data_dir = PathBuf::from(shellexpand::tilde(data_dir).to_string());
        std::fs::create_dir_all(&data_dir)?;

        let manager = Self {
            data_dir,
            gid_to_job: RwLock::new(HashMap::new()),
            job_to_gid: RwLock::new(HashMap::new()),
        };

        // TODO: Load persisted session on startup

        Ok(manager)
    }

    /// Register a GID <-> Job ID mapping.
    pub async fn register(&self, gid: String, job_id: i64) {
        self.gid_to_job.write().await.insert(gid.clone(), job_id);
        self.job_to_gid.write().await.insert(job_id, gid);
    }

    /// Remove a mapping by GID.
    pub async fn remove_by_gid(&self, gid: &str) {
        if let Some(job_id) = self.gid_to_job.write().await.remove(gid) {
            self.job_to_gid.write().await.remove(&job_id);
        }
    }

    /// Remove a mapping by job ID.
    pub async fn remove_by_job(&self, job_id: i64) {
        if let Some(gid) = self.job_to_gid.write().await.remove(&job_id) {
            self.gid_to_job.write().await.remove(&gid);
        }
    }

    /// Get job ID for a GID.
    pub async fn get_job_id(&self, gid: &str) -> Option<i64> {
        self.gid_to_job.read().await.get(gid).copied()
    }

    /// Get GID for a job ID.
    pub async fn get_gid(&self, job_id: i64) -> Option<String> {
        self.job_to_gid.read().await.get(&job_id).cloned()
    }

    /// Persist session to disk.
    pub async fn save(&self) -> Result<()> {
        let mappings: Vec<(String, i64)> = self
            .gid_to_job
            .read()
            .await
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        let data = SessionData { mappings };
        let path = self.data_dir.join("session.bin");
        let bytes = bincode::serialize(&data)?;
        tokio::fs::write(path, bytes).await?;

        Ok(())
    }

    /// Load session from disk.
    pub async fn load(&self) -> Result<()> {
        let path = self.data_dir.join("session.bin");

        if !path.exists() {
            return Ok(());
        }

        let bytes = tokio::fs::read(&path).await?;
        let data: SessionData = bincode::deserialize(&bytes)?;

        let mut gid_to_job = self.gid_to_job.write().await;
        let mut job_to_gid = self.job_to_gid.write().await;

        for (gid, job_id) in data.mappings {
            gid_to_job.insert(gid.clone(), job_id);
            job_to_gid.insert(job_id, gid);
        }

        Ok(())
    }
}

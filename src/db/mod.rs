//! Database module
//!
//! SQLite storage for job metadata, history, and state.

mod queries;
mod state;

pub use queries::*;
pub use state::*;

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::config::Config;
use crate::queue::{Job, JobStatus};

/// Database connection wrapper.
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection.
    pub async fn new(db_path: &str) -> Result<Self> {
        let db_path = shellexpand::tilde(db_path).to_string();
        let path = PathBuf::from(&db_path);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("Failed to connect to database")?;

        let db = Self { pool };
        db.run_migrations().await?;

        Ok(db)
    }

    /// Run database migrations.
    async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'queued',
                gid TEXT,
                filename TEXT,
                dir TEXT,
                output_path TEXT,
                total_size INTEGER DEFAULT 0,
                downloaded INTEGER DEFAULT 0,
                progress REAL DEFAULT 0.0,
                speed INTEGER DEFAULT 0,
                category TEXT,
                tags TEXT DEFAULT '[]',
                headers TEXT DEFAULT '[]',
                referer TEXT,
                user_agent TEXT,
                error TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                completed_at INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_jobs_gid ON jobs(gid)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Insert a new job.
    pub async fn insert_job(&self, job: &Job) -> Result<i64> {
        let tags_json = serde_json::to_string(&job.tags)?;
        let headers_json = serde_json::to_string(&job.headers)?;

        let result = sqlx::query(
            r#"
            INSERT INTO jobs (url, status, filename, dir, category, tags, headers, referer, user_agent, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&job.url)
        .bind(job.status.to_string())
        .bind(&job.filename)
        .bind(&job.dir)
        .bind(&job.category)
        .bind(&tags_json)
        .bind(&headers_json)
        .bind(&job.referer)
        .bind(&job.user_agent)
        .bind(job.created_at)
        .bind(job.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get a job by ID.
    pub async fn get_job(&self, id: i64) -> Result<Job> {
        let row = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .context("Job not found")?;

        row.into_job()
    }

    /// Update job status.
    pub async fn update_job_status(&self, id: i64, status: JobStatus) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let completed_at = if status == JobStatus::Complete {
            Some(now)
        } else {
            None
        };

        sqlx::query("UPDATE jobs SET status = ?, updated_at = ?, completed_at = COALESCE(?, completed_at) WHERE id = ?")
            .bind(status.to_string())
            .bind(now)
            .bind(completed_at)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update job GID.
    pub async fn update_job_gid(&self, id: i64, gid: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query("UPDATE jobs SET gid = ?, updated_at = ? WHERE id = ?")
            .bind(gid)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete a job.
    pub async fn delete_job(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM jobs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get jobs by status.
    pub async fn get_jobs_by_status(&self, status: JobStatus) -> Result<Vec<Job>> {
        let rows = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs WHERE status = ? ORDER BY created_at")
            .bind(status.to_string())
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(|r| r.into_job()).collect()
    }

    /// List jobs with optional filters.
    pub async fn list_jobs(&self, status: Option<&str>, limit: Option<usize>) -> Result<Vec<Job>> {
        let limit = limit.unwrap_or(100) as i64;

        let rows = if let Some(status) = status {
            sqlx::query_as::<_, JobRow>(
                "SELECT * FROM jobs WHERE status = ? ORDER BY created_at DESC LIMIT ?",
            )
            .bind(status)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, JobRow>(
                "SELECT * FROM jobs ORDER BY created_at DESC LIMIT ?",
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(|r| r.into_job()).collect()
    }

    /// Get next queued job.
    pub async fn get_next_queued_job(&self) -> Result<Option<Job>> {
        let row = sqlx::query_as::<_, JobRow>(
            "SELECT * FROM jobs WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_job()?)),
            None => Ok(None),
        }
    }
}

/// Initialize database from config.
pub async fn init(config: &Config) -> Result<Arc<Database>> {
    let db_path = format!("{}/rustaria.db", config.general.data_dir);
    let db = Database::new(&db_path).await?;
    Ok(Arc::new(db))
}

/// Database row for jobs table.
#[derive(Debug, sqlx::FromRow)]
struct JobRow {
    id: i64,
    url: String,
    status: String,
    gid: Option<String>,
    filename: Option<String>,
    dir: Option<String>,
    output_path: Option<String>,
    total_size: i64,
    downloaded: i64,
    progress: f64,
    speed: i64,
    category: Option<String>,
    tags: String,
    headers: String,
    referer: Option<String>,
    user_agent: Option<String>,
    error: Option<String>,
    created_at: i64,
    updated_at: i64,
    completed_at: Option<i64>,
}

impl JobRow {
    fn into_job(self) -> Result<Job> {
        let tags: Vec<String> = serde_json::from_str(&self.tags).unwrap_or_default();
        let headers: Vec<String> = serde_json::from_str(&self.headers).unwrap_or_default();

        Ok(Job {
            id: self.id,
            url: self.url,
            status: self.status.parse()?,
            gid: self.gid,
            filename: self.filename,
            dir: self.dir,
            output_path: self.output_path,
            total_size: self.total_size as u64,
            downloaded: self.downloaded as u64,
            progress: self.progress,
            speed: self.speed as u64,
            category: self.category,
            tags,
            headers,
            referer: self.referer,
            user_agent: self.user_agent,
            error: self.error,
            created_at: self.created_at,
            updated_at: self.updated_at,
            completed_at: self.completed_at,
        })
    }
}

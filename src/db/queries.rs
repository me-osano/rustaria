//! Database query helpers.

use anyhow::Result;
use super::Database;

impl Database {
    /// Get total download count.
    pub async fn count_downloads(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM jobs")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0)
    }

    /// Get download count by status.
    pub async fn count_by_status(&self, status: &str) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM jobs WHERE status = ?")
            .bind(status)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0)
    }

    /// Get total bytes downloaded.
    pub async fn total_downloaded(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COALESCE(SUM(downloaded), 0) FROM jobs")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0)
    }

    /// Search jobs by URL.
    pub async fn search_by_url(&self, query: &str) -> Result<Vec<super::Job>> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, super::JobRow>(
            "SELECT * FROM jobs WHERE url LIKE ? ORDER BY created_at DESC LIMIT 100",
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_job()).collect()
    }

    /// Get jobs by category.
    pub async fn get_by_category(&self, category: &str) -> Result<Vec<super::Job>> {
        let rows = sqlx::query_as::<_, super::JobRow>(
            "SELECT * FROM jobs WHERE category = ? ORDER BY created_at DESC",
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_job()).collect()
    }

    /// Get recent downloads.
    pub async fn get_recent(&self, limit: i64) -> Result<Vec<super::Job>> {
        let rows = sqlx::query_as::<_, super::JobRow>(
            "SELECT * FROM jobs ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_job()).collect()
    }

    /// Clear completed downloads older than N days.
    pub async fn clear_old_completed(&self, days: i64) -> Result<i64> {
        let cutoff = chrono::Utc::now().timestamp() - (days * 24 * 60 * 60);

        let result = sqlx::query(
            "DELETE FROM jobs WHERE status = 'complete' AND completed_at < ?",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Update job progress.
    pub async fn update_progress(
        &self,
        id: i64,
        downloaded: u64,
        total: u64,
        speed: u64,
    ) -> Result<()> {
        let progress = if total > 0 {
            (downloaded as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            "UPDATE jobs SET downloaded = ?, total_size = ?, progress = ?, speed = ?, updated_at = ? WHERE id = ?",
        )
        .bind(downloaded as i64)
        .bind(total as i64)
        .bind(progress)
        .bind(speed as i64)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

//! Hook system
//!
//! User-defined shell hooks for download events.

use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

use crate::queue::Job;

/// Run a hook script.
pub async fn run(script: &str, job: &Job) -> Result<()> {
    tracing::info!("Running hook: {}", script);

    // Set environment variables for the hook
    let status = Command::new("sh")
        .arg("-c")
        .arg(script)
        .env("FERRUM_JOB_ID", job.id.to_string())
        .env("FERRUM_URL", &job.url)
        .env("FERRUM_STATUS", job.status.to_string())
        .env(
            "FERRUM_FILENAME",
            job.filename.as_deref().unwrap_or(""),
        )
        .env(
            "FERRUM_OUTPUT_PATH",
            job.output_path.as_deref().unwrap_or(""),
        )
        .env("FERRUM_SIZE", job.total_size.to_string())
        .env(
            "FERRUM_CATEGORY",
            job.category.as_deref().unwrap_or(""),
        )
        .env("FERRUM_TAGS", job.tags.join(","))
        .env(
            "FERRUM_ERROR",
            job.error.as_deref().unwrap_or(""),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        tracing::warn!("Hook exited with status {}", status);
    }

    Ok(())
}

/// Run the on_complete hook.
pub async fn on_complete(script: &str, job: &Job) -> Result<()> {
    run(script, job).await
}

/// Run the on_error hook.
pub async fn on_error(script: &str, job: &Job) -> Result<()> {
    run(script, job).await
}

/// Run the on_queue_empty hook.
pub async fn on_queue_empty(script: &str) -> Result<()> {
    tracing::info!("Running on_queue_empty hook: {}", script);

    let status = Command::new("sh")
        .arg("-c")
        .arg(script)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        tracing::warn!("Hook exited with status {}", status);
    }

    Ok(())
}

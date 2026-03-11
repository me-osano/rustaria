//! Post-processing layer
//!
//! File organization, FFmpeg merging, archive extraction, and hooks.

pub mod extractor;
pub mod ffmpeg;
pub mod hooks;
pub mod organizer;
pub mod rules;

use anyhow::Result;

use crate::config::Config;
use crate::queue::Job;

/// Post-process a completed download.
pub async fn process(job: &Job, config: &Config) -> Result<()> {
    let output_path = match &job.output_path {
        Some(path) => path.clone(),
        None => return Ok(()), // No file to process
    };

    // Auto-organize
    if config.postprocess.auto_organize {
        if let Err(e) = organizer::organize(&output_path, config).await {
            tracing::error!("Failed to organize file: {}", e);
        }
    }

    // Auto-extract archives
    if config.postprocess.auto_extract {
        if let Err(e) = extractor::extract(&output_path, config).await {
            tracing::error!("Failed to extract archive: {}", e);
        }
    }

    // Merge HLS/DASH with FFmpeg
    if config.postprocess.merge_media {
        if let Err(e) = ffmpeg::merge_if_needed(&output_path, config).await {
            tracing::error!("Failed to merge media: {}", e);
        }
    }

    // Run completion hook
    if let Some(ref hook) = config.hooks.on_complete {
        if let Err(e) = hooks::run(hook, job).await {
            tracing::error!("Failed to run on_complete hook: {}", e);
        }
    }

    Ok(())
}

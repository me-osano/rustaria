//! FFmpeg bridge
//!
//! Merges HLS/DASH fragments and remuxes media files.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::config::Config;

/// Merge media if needed (HLS/DASH).
pub async fn merge_if_needed(file_path: &str, config: &Config) -> Result<()> {
    let path = Path::new(file_path);

    // Check if this is an HLS/DASH file
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if extension != "m3u8" && extension != "mpd" {
        return Ok(());
    }

    // Build output path
    let output_path = path.with_extension("mp4");

    tracing::info!("Merging {} -> {}", path.display(), output_path.display());

    merge(file_path, output_path.to_str().unwrap(), config).await?;

    // Optionally delete original
    // tokio::fs::remove_file(path).await?;

    Ok(())
}

/// Merge/remux a media file using FFmpeg.
pub async fn merge(input: &str, output: &str, config: &Config) -> Result<()> {
    let ffmpeg = if config.postprocess.ffmpeg_path.is_empty() {
        "ffmpeg"
    } else {
        &config.postprocess.ffmpeg_path
    };

    let status = Command::new(ffmpeg)
        .args([
            "-i",
            input,
            "-c",
            "copy", // Copy streams without re-encoding
            "-y",   // Overwrite output
            output,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await
        .context("Failed to run FFmpeg")?;

    if !status.success() {
        anyhow::bail!("FFmpeg exited with status {}", status);
    }

    Ok(())
}

/// Remux a file to a different container format.
pub async fn remux(input: &str, output: &str, config: &Config) -> Result<()> {
    merge(input, output, config).await
}

/// Extract audio from a video file.
pub async fn extract_audio(input: &str, output: &str, config: &Config) -> Result<()> {
    let ffmpeg = if config.postprocess.ffmpeg_path.is_empty() {
        "ffmpeg"
    } else {
        &config.postprocess.ffmpeg_path
    };

    let status = Command::new(ffmpeg)
        .args([
            "-i",
            input,
            "-vn",      // No video
            "-acodec",
            "copy",     // Copy audio stream
            "-y",
            output,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await
        .context("Failed to run FFmpeg")?;

    if !status.success() {
        anyhow::bail!("FFmpeg exited with status {}", status);
    }

    Ok(())
}

/// Get media info using ffprobe.
pub async fn get_info(file_path: &str) -> Result<MediaInfo> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            file_path,
        ])
        .output()
        .await
        .context("Failed to run ffprobe")?;

    let info: MediaInfo = serde_json::from_slice(&output.stdout)
        .context("Failed to parse ffprobe output")?;

    Ok(info)
}

/// Media information from ffprobe.
#[derive(Debug, serde::Deserialize)]
pub struct MediaInfo {
    pub format: Option<FormatInfo>,
    pub streams: Option<Vec<StreamInfo>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FormatInfo {
    pub filename: Option<String>,
    pub format_name: Option<String>,
    pub duration: Option<String>,
    pub size: Option<String>,
    pub bit_rate: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct StreamInfo {
    pub index: i32,
    pub codec_name: Option<String>,
    pub codec_type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub sample_rate: Option<String>,
    pub channels: Option<i32>,
}

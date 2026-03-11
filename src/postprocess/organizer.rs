//! File organizer
//!
//! Categorizes downloads into folders based on file type.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::Config;

/// Organize a file into the appropriate category folder.
pub async fn organize(file_path: &str, config: &Config) -> Result<()> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Ok(());
    }

    // Get file extension
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    // Find matching category
    let category = config
        .postprocess
        .categories
        .iter()
        .find(|(_, exts)| exts.iter().any(|e| e.to_lowercase() == extension))
        .map(|(cat, _)| cat.clone());

    let category = match category {
        Some(c) => c,
        None => return Ok(()), // No category matched
    };

    // Build destination path
    let download_dir = shellexpand::tilde(&config.general.download_dir);
    let dest_dir = PathBuf::from(download_dir.as_ref()).join(&category);

    // Create category directory if needed
    tokio::fs::create_dir_all(&dest_dir).await?;

    // Move file to category folder
    let file_name = path.file_name().unwrap();
    let dest_path = dest_dir.join(file_name);

    // Don't move if already in the right place
    if path == dest_path {
        return Ok(());
    }

    tracing::info!(
        "Organizing {} -> {}",
        path.display(),
        dest_path.display()
    );

    tokio::fs::rename(path, &dest_path).await?;

    Ok(())
}

/// Get the category for a file based on its extension.
pub fn get_category(path: &Path, config: &Config) -> Option<String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())?;

    config
        .postprocess
        .categories
        .iter()
        .find(|(_, exts)| exts.iter().any(|e| e.to_lowercase() == extension))
        .map(|(cat, _)| cat.clone())
}

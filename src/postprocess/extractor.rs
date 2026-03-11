//! Archive extractor
//!
//! Auto-extracts ZIP, tar.gz archives.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::config::Config;

/// Extract an archive if it's a supported format.
pub async fn extract(file_path: &str, _config: &Config) -> Result<()> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Ok(());
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "zip" => extract_zip(path).await,
        "gz" => {
            // Check if it's .tar.gz
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if stem.ends_with(".tar") {
                extract_tar_gz(path).await
            } else {
                Ok(()) // Plain .gz, skip for now
            }
        }
        "tar" => extract_tar(path).await,
        _ => Ok(()), // Unsupported format
    }
}

/// Extract a ZIP archive.
async fn extract_zip(path: &Path) -> Result<()> {
    let file = File::open(path).context("Failed to open ZIP file")?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))?;

    let dest_dir = path.parent().unwrap_or(Path::new("."));

    tracing::info!("Extracting ZIP: {} -> {}", path.display(), dest_dir.display());

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

/// Extract a tar archive.
async fn extract_tar(path: &Path) -> Result<()> {
    let file = File::open(path).context("Failed to open tar file")?;
    let mut archive = tar::Archive::new(file);

    let dest_dir = path.parent().unwrap_or(Path::new("."));

    tracing::info!("Extracting tar: {} -> {}", path.display(), dest_dir.display());

    archive.unpack(dest_dir)?;

    Ok(())
}

/// Extract a tar.gz archive.
async fn extract_tar_gz(path: &Path) -> Result<()> {
    let file = File::open(path).context("Failed to open tar.gz file")?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);

    let dest_dir = path.parent().unwrap_or(Path::new("."));

    tracing::info!("Extracting tar.gz: {} -> {}", path.display(), dest_dir.display());

    archive.unpack(dest_dir)?;

    Ok(())
}

/// Check if a file is a supported archive format.
pub fn is_archive(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    matches!(extension.as_str(), "zip" | "tar" | "gz" | "rar" | "7z")
}

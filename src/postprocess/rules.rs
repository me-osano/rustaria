//! Organization rules.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A rule for organizing files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Rule name.
    pub name: String,

    /// File extensions to match.
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Regex pattern to match filename.
    #[serde(default)]
    pub pattern: Option<String>,

    /// Minimum file size in bytes.
    #[serde(default)]
    pub min_size: Option<u64>,

    /// Maximum file size in bytes.
    #[serde(default)]
    pub max_size: Option<u64>,

    /// Destination directory (relative to download_dir).
    pub destination: String,

    /// Whether to create subdirectories by date.
    #[serde(default)]
    pub date_subdirs: bool,
}

impl Rule {
    /// Check if a file matches this rule.
    pub fn matches(&self, path: &Path, size: u64) -> bool {
        // Check extension
        if !self.extensions.is_empty() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();

            if !self.extensions.iter().any(|e| e.to_lowercase() == ext) {
                return false;
            }
        }

        // Check pattern
        if let Some(ref pattern) = self.pattern {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            if let Ok(re) = regex::Regex::new(pattern) {
                if !re.is_match(filename) {
                    return false;
                }
            }
        }

        // Check size constraints
        if let Some(min) = self.min_size {
            if size < min {
                return false;
            }
        }

        if let Some(max) = self.max_size {
            if size > max {
                return false;
            }
        }

        true
    }
}

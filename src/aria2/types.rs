//! Shared types for aria2 communication.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for aria2.addUri.
#[derive(Debug, Default, Serialize)]
pub struct AddUriOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub referer: Option<String>,

    #[serde(rename = "user-agent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    #[serde(rename = "max-connection-per-server", skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u32>,

    #[serde(rename = "split", skip_serializing_if = "Option::is_none")]
    pub split: Option<u32>,

    #[serde(rename = "max-download-limit", skip_serializing_if = "Option::is_none")]
    pub max_download_limit: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// Download status returned by aria2.tellStatus.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStatus {
    pub gid: String,
    pub status: DownloadState,
    pub total_length: String,
    pub completed_length: String,
    pub upload_length: String,
    pub download_speed: String,
    pub upload_speed: String,
    pub connections: String,

    #[serde(default)]
    pub error_code: Option<String>,

    #[serde(default)]
    pub error_message: Option<String>,

    #[serde(default)]
    pub files: Vec<DownloadFile>,

    #[serde(default)]
    pub dir: String,

    #[serde(default)]
    pub bittorrent: Option<BittorrentInfo>,
}

impl DownloadStatus {
    /// Get total length as u64.
    pub fn total_bytes(&self) -> u64 {
        self.total_length.parse().unwrap_or(0)
    }

    /// Get completed length as u64.
    pub fn completed_bytes(&self) -> u64 {
        self.completed_length.parse().unwrap_or(0)
    }

    /// Get download speed as u64.
    pub fn speed(&self) -> u64 {
        self.download_speed.parse().unwrap_or(0)
    }

    /// Calculate progress percentage.
    pub fn progress(&self) -> f64 {
        let total = self.total_bytes();
        if total == 0 {
            return 0.0;
        }
        (self.completed_bytes() as f64 / total as f64) * 100.0
    }
}

/// Download state enum.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadState {
    Active,
    Waiting,
    Paused,
    Error,
    Complete,
    Removed,
}

/// File information within a download.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadFile {
    pub index: String,
    pub path: String,
    pub length: String,
    pub completed_length: String,
    pub selected: String,
    #[serde(default)]
    pub uris: Vec<FileUri>,
}

/// URI information for a file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileUri {
    pub uri: String,
    pub status: String,
}

/// BitTorrent-specific information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BittorrentInfo {
    pub announce_list: Option<Vec<Vec<String>>>,
    pub comment: Option<String>,
    pub creation_date: Option<u64>,
    pub mode: Option<String>,
    pub info: Option<BittorrentMetaInfo>,
}

/// BitTorrent metadata info.
#[derive(Debug, Clone, Deserialize)]
pub struct BittorrentMetaInfo {
    pub name: Option<String>,
}

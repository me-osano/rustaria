//! Configuration schema definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub aria2: Aria2Config,
    pub scheduler: SchedulerConfig,
    pub postprocess: PostprocessConfig,
    pub hooks: HooksConfig,
    pub ui: UiConfig,
    pub notifications: NotificationsConfig,
    pub integration: IntegrationConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            aria2: Aria2Config::default(),
            scheduler: SchedulerConfig::default(),
            postprocess: PostprocessConfig::default(),
            hooks: HooksConfig::default(),
            ui: UiConfig::default(),
            notifications: NotificationsConfig::default(),
            integration: IntegrationConfig::default(),
        }
    }
}

/// General settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub download_dir: String,
    pub max_concurrent: usize,
    pub clipboard_monitor: bool,
    pub clipboard_patterns: Vec<String>,
    pub data_dir: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            download_dir: "~/Downloads".to_string(),
            max_concurrent: 5,
            clipboard_monitor: false,
            clipboard_patterns: vec![],
            data_dir: "~/.local/share/rustaria".to_string(),
        }
    }
}

/// aria2 connection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Aria2Config {
    pub rpc_url: String,
    pub rpc_secret: String,
    pub auto_start: bool,
    pub binary_path: String,
    pub extra_args: Vec<String>,
}

impl Default for Aria2Config {
    fn default() -> Self {
        Self {
            rpc_url: "http://127.0.0.1:6800/jsonrpc".to_string(),
            rpc_secret: String::new(),
            auto_start: true,
            binary_path: String::new(),
            extra_args: vec![
                "--max-connection-per-server=16".to_string(),
                "--split=16".to_string(),
            ],
        }
    }
}

/// Scheduler settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub active_window: Option<String>,
    pub bandwidth_limit: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active_window: None,
            bandwidth_limit: 0,
        }
    }
}

/// Post-processing settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PostprocessConfig {
    pub auto_organize: bool,
    pub categories: HashMap<String, Vec<String>>,
    pub auto_extract: bool,
    pub merge_media: bool,
    pub ffmpeg_path: String,
}

impl Default for PostprocessConfig {
    fn default() -> Self {
        let mut categories = HashMap::new();
        categories.insert(
            "video".to_string(),
            vec!["mp4", "mkv", "avi", "mov", "webm"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        categories.insert(
            "audio".to_string(),
            vec!["mp3", "flac", "wav", "aac", "ogg"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        categories.insert(
            "documents".to_string(),
            vec!["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        categories.insert(
            "archives".to_string(),
            vec!["zip", "tar", "gz", "rar", "7z"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        Self {
            auto_organize: true,
            categories,
            auto_extract: false,
            merge_media: true,
            ffmpeg_path: String::new(),
        }
    }
}

/// Hook scripts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HooksConfig {
    pub on_complete: Option<String>,
    pub on_error: Option<String>,
    pub on_queue_empty: Option<String>,
}

/// UI settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub theme: String,
    pub refresh_rate: u64,
    pub show_graph: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            refresh_rate: 250,
            show_graph: true,
        }
    }
}

/// Notification settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationsConfig {
    pub enabled: bool,
    pub on_complete: bool,
    pub on_error: bool,
    pub on_queue_empty: bool,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            on_complete: true,
            on_error: true,
            on_queue_empty: false,
        }
    }
}

/// Integration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IntegrationConfig {
    pub native_messaging: bool,
    pub media_sniffer: bool,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            native_messaging: true,
            media_sniffer: false,
        }
    }
}

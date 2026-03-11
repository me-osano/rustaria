//! aria2 JSON-RPC method definitions
//!
//! Provides type-safe wrappers for aria2 JSON-RPC 2.0 methods.
//! See: https://aria2.github.io/manual/en/html/aria2c.html#methods

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// aria2 RPC method names
pub mod method {
    pub const ADD_URI: &str = "aria2.addUri";
    pub const ADD_TORRENT: &str = "aria2.addTorrent";
    pub const ADD_METALINK: &str = "aria2.addMetalink";
    pub const REMOVE: &str = "aria2.remove";
    pub const FORCE_REMOVE: &str = "aria2.forceRemove";
    pub const PAUSE: &str = "aria2.pause";
    pub const PAUSE_ALL: &str = "aria2.pauseAll";
    pub const FORCE_PAUSE: &str = "aria2.forcePause";
    pub const FORCE_PAUSE_ALL: &str = "aria2.forcePauseAll";
    pub const UNPAUSE: &str = "aria2.unpause";
    pub const UNPAUSE_ALL: &str = "aria2.unpauseAll";
    pub const TELL_STATUS: &str = "aria2.tellStatus";
    pub const GET_URIS: &str = "aria2.getUris";
    pub const GET_FILES: &str = "aria2.getFiles";
    pub const GET_PEERS: &str = "aria2.getPeers";
    pub const GET_SERVERS: &str = "aria2.getServers";
    pub const TELL_ACTIVE: &str = "aria2.tellActive";
    pub const TELL_WAITING: &str = "aria2.tellWaiting";
    pub const TELL_STOPPED: &str = "aria2.tellStopped";
    pub const CHANGE_POSITION: &str = "aria2.changePosition";
    pub const CHANGE_URI: &str = "aria2.changeUri";
    pub const GET_OPTION: &str = "aria2.getOption";
    pub const CHANGE_OPTION: &str = "aria2.changeOption";
    pub const GET_GLOBAL_OPTION: &str = "aria2.getGlobalOption";
    pub const CHANGE_GLOBAL_OPTION: &str = "aria2.changeGlobalOption";
    pub const GET_GLOBAL_STAT: &str = "aria2.getGlobalStat";
    pub const PURGE_DOWNLOAD_RESULT: &str = "aria2.purgeDownloadResult";
    pub const REMOVE_DOWNLOAD_RESULT: &str = "aria2.removeDownloadResult";
    pub const GET_VERSION: &str = "aria2.getVersion";
    pub const GET_SESSION_INFO: &str = "aria2.getSessionInfo";
    pub const SHUTDOWN: &str = "aria2.shutdown";
    pub const FORCE_SHUTDOWN: &str = "aria2.forceShutdown";
    pub const SAVE_SESSION: &str = "aria2.saveSession";
    pub const MULTICALL: &str = "system.multicall";
    pub const LIST_METHODS: &str = "system.listMethods";
    pub const LIST_NOTIFICATIONS: &str = "system.listNotifications";
}

/// Options for adding a new download
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddUriOptions {
    /// Directory to save the downloaded file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,

    /// File name of the downloaded file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,

    /// HTTP headers to send
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<Vec<String>>,

    /// Referer URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referer: Option<String>,

    /// User agent string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Maximum download speed in bytes/sec (0 = unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_download_limit: Option<String>,

    /// Maximum number of connections per server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connection_per_server: Option<u32>,

    /// Minimum split size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_split_size: Option<String>,

    /// Number of splits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<u32>,

    /// Cookie data or file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_cookies: Option<String>,

    /// Additional custom options
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Position adjustment mode for changePosition
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionMode {
    /// Set position relative to the beginning of the queue
    PosSet,
    /// Set position relative to the current position
    PosCur,
    /// Set position relative to the end of the queue
    PosEnd,
}

/// Global statistics response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalStat {
    /// Overall download speed in bytes/sec
    pub download_speed: String,
    /// Overall upload speed in bytes/sec
    pub upload_speed: String,
    /// Number of active downloads
    pub num_active: String,
    /// Number of waiting downloads
    pub num_waiting: String,
    /// Number of stopped downloads
    pub num_stopped: String,
    /// Number of stopped downloads in the current session
    pub num_stopped_total: String,
}

/// Version information response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    /// aria2 version string
    pub version: String,
    /// List of enabled features
    pub enabled_features: Vec<String>,
}

/// Session information response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    /// Session ID
    pub session_id: String,
}

/// URI status in a download
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UriStatus {
    /// URI string
    pub uri: String,
    /// Status: "used" or "waiting"
    pub status: String,
}

/// File information in a download
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    /// Index of the file (1-based)
    pub index: String,
    /// File path
    pub path: String,
    /// File size in bytes
    pub length: String,
    /// Completed length in bytes
    pub completed_length: String,
    /// true if this file is selected for download
    pub selected: String,
    /// List of URIs for this file
    #[serde(default)]
    pub uris: Vec<UriStatus>,
}

/// Server information for a download
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// Index of the file (1-based)
    pub index: String,
    /// List of servers
    pub servers: Vec<Server>,
}

/// Individual server details
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    /// URI of the server
    pub uri: String,
    /// Current download speed from this server
    pub current_uri: String,
    /// Download speed in bytes/sec
    pub download_speed: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_uri_options_serialization() {
        let options = AddUriOptions {
            dir: Some("/downloads".to_string()),
            out: Some("test.zip".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&options).unwrap();
        assert_eq!(json["dir"], "/downloads");
        assert_eq!(json["out"], "test.zip");
    }

    #[test]
    fn test_global_stat_deserialization() {
        let json = r#"{
            "downloadSpeed": "1024",
            "uploadSpeed": "512",
            "numActive": "2",
            "numWaiting": "5",
            "numStopped": "10",
            "numStoppedTotal": "100"
        }"#;

        let stat: GlobalStat = serde_json::from_str(json).unwrap();
        assert_eq!(stat.download_speed, "1024");
        assert_eq!(stat.num_active, "2");
    }
}

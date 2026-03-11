//! Media sniffer
//!
//! Detects HLS/DASH manifests and direct media URLs.

use anyhow::Result;
use regex::Regex;

/// Known media URL patterns.
pub struct MediaSniffer {
    hls_pattern: Regex,
    dash_pattern: Regex,
    direct_media_pattern: Regex,
}

impl MediaSniffer {
    /// Create a new media sniffer.
    pub fn new() -> Result<Self> {
        Ok(Self {
            hls_pattern: Regex::new(r"\.m3u8(\?.*)?$")?,
            dash_pattern: Regex::new(r"\.mpd(\?.*)?$")?,
            direct_media_pattern: Regex::new(
                r"\.(mp4|mkv|webm|avi|mov|mp3|flac|wav|aac)(\?.*)?$",
            )?,
        })
    }

    /// Detect media type from URL.
    pub fn detect(&self, url: &str) -> Option<MediaType> {
        if self.hls_pattern.is_match(url) {
            Some(MediaType::Hls)
        } else if self.dash_pattern.is_match(url) {
            Some(MediaType::Dash)
        } else if self.direct_media_pattern.is_match(url) {
            Some(MediaType::Direct)
        } else {
            None
        }
    }

    /// Extract media URLs from HTML content.
    pub fn extract_from_html(&self, html: &str) -> Vec<String> {
        let mut urls = Vec::new();

        // Simple regex to find URLs - could be improved with proper HTML parsing
        let url_pattern = Regex::new(r#"(?:src|href)=["']([^"']+)["']"#).unwrap();

        for cap in url_pattern.captures_iter(html) {
            if let Some(url) = cap.get(1) {
                let url_str = url.as_str();
                if self.detect(url_str).is_some() {
                    urls.push(url_str.to_string());
                }
            }
        }

        urls
    }
}

impl Default for MediaSniffer {
    fn default() -> Self {
        Self::new().expect("Failed to create MediaSniffer")
    }
}

/// Detected media type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    /// HLS manifest (.m3u8)
    Hls,
    /// DASH manifest (.mpd)
    Dash,
    /// Direct media file
    Direct,
}

impl MediaType {
    /// Check if this type requires FFmpeg for processing.
    pub fn requires_ffmpeg(&self) -> bool {
        matches!(self, Self::Hls | Self::Dash)
    }
}

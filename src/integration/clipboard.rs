//! Clipboard monitoring
//!
//! Watches clipboard for URLs matching configured patterns.

use anyhow::Result;
use arboard::Clipboard;
use regex::Regex;
use std::time::Duration;

use crate::queue::JobQueue;

/// Run the clipboard monitor.
pub async fn run(patterns: Vec<String>, queue: JobQueue) -> Result<()> {
    tracing::info!("Starting clipboard monitor");

    // Compile regex patterns
    let regexes: Vec<Regex> = patterns
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect();

    if regexes.is_empty() {
        tracing::warn!("No valid clipboard patterns configured");
        return Ok(());
    }

    let mut clipboard = Clipboard::new()?;
    let mut last_content = String::new();

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check clipboard content
        let content = match clipboard.get_text() {
            Ok(text) => text,
            Err(_) => continue,
        };

        // Skip if same as last check
        if content == last_content {
            continue;
        }
        last_content = content.clone();

        // Check if content matches any pattern
        for regex in &regexes {
            if regex.is_match(&content) {
                tracing::info!("Clipboard URL matched: {}", content);

                // Add to queue
                if let Err(e) = queue
                    .add(&content, None, None, Some("clipboard".to_string()), vec![])
                    .await
                {
                    tracing::error!("Failed to add clipboard URL: {}", e);
                }

                break;
            }
        }
    }
}

/// Check if a string looks like a URL.
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("magnet:")
}

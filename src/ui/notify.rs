//! Desktop notifications.

use anyhow::Result;
use notify_rust::{Notification, Timeout};

/// Send a download complete notification.
pub fn download_complete(filename: &str) -> Result<()> {
    Notification::new()
        .summary("Download Complete")
        .body(&format!("{} has finished downloading", filename))
        .icon("dialog-information")
        .timeout(Timeout::Milliseconds(5000))
        .show()?;

    Ok(())
}

/// Send a download error notification.
pub fn download_error(filename: &str, error: &str) -> Result<()> {
    Notification::new()
        .summary("Download Failed")
        .body(&format!("{}: {}", filename, error))
        .icon("dialog-error")
        .timeout(Timeout::Milliseconds(10000))
        .show()?;

    Ok(())
}

/// Send a queue empty notification.
pub fn queue_empty() -> Result<()> {
    Notification::new()
        .summary("All Downloads Complete")
        .body("The download queue is now empty")
        .icon("dialog-information")
        .timeout(Timeout::Milliseconds(5000))
        .show()?;

    Ok(())
}

/// Send a generic notification.
pub fn notify(title: &str, body: &str) -> Result<()> {
    Notification::new()
        .summary(title)
        .body(body)
        .icon("dialog-information")
        .timeout(Timeout::Milliseconds(5000))
        .show()?;

    Ok(())
}

//! Configuration module
//!
//! Declarative TOML config, runtime reload support.

mod schema;

pub use schema::*;

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Load configuration from file.
pub fn load(path: &Option<PathBuf>) -> Result<Config> {
    let config_path = path.clone().unwrap_or_else(default_config_path);

    if !config_path.exists() {
        tracing::info!("Config file not found, using defaults");
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

    tracing::info!("Loaded configuration from {:?}", config_path);
    Ok(config)
}

/// Get the default configuration file path.
pub fn default_config_path() -> PathBuf {
    // Check XDG config first
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg_config).join("rustaria/config.toml");
    }

    // Fall back to ~/.config
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config/rustaria/config.toml");
    }

    // Last resort: current directory
    PathBuf::from("config.toml")
}

/// Save configuration to file.
pub fn save(config: &Config, path: &Option<PathBuf>) -> Result<()> {
    let config_path = path.clone().unwrap_or_else(default_config_path);

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    std::fs::write(&config_path, content)?;

    tracing::info!("Saved configuration to {:?}", config_path);
    Ok(())
}

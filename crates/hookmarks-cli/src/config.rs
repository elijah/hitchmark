//! Configuration file handling.
//!
//! Reads ~/.config/hookmarks/config.toml and provides settings for the CLI.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Path to the link store database
    pub store_path: PathBuf,
    /// Whether to open files after linking
    pub auto_open: bool,
    /// Default note template
    pub note_template: Option<String>,
}

/// Resolve the hookmarks config directory, returning an error if HOME is not set.
fn hookmarks_config_dir() -> anyhow::Result<PathBuf> {
    dirs::config_dir()
        .map(|d| d.join("hookmarks"))
        .ok_or_else(|| anyhow::anyhow!(
            "Could not determine config directory. \
             Ensure $HOME (or $XDG_CONFIG_HOME) is set."
        ))
}

impl Default for Config {
    fn default() -> Self {
        // Fall back to a sensible path even if dirs::config_dir fails
        let store_path = dirs::config_dir()
            .map(|d| d.join("hookmarks").join("store.db"))
            .unwrap_or_else(|| PathBuf::from(".hookmarks/store.db"));

        Config {
            store_path,
            auto_open: false,
            note_template: None,
        }
    }
}

impl Config {
    /// Load config from file, or return defaults if file doesn't exist.
    pub fn load() -> anyhow::Result<Self> {
        let config_path = hookmarks_config_dir()?.join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Config::default())
        }
    }

    /// Ensure config directory exists.
    pub fn ensure_dir(&self) -> anyhow::Result<()> {
        let hookmarks_dir = hookmarks_config_dir()?;

        if !hookmarks_dir.exists() {
            std::fs::create_dir_all(&hookmarks_dir)?;
        }

        // Ensure parent of store_path exists
        if let Some(parent) = self.store_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        Ok(())
    }
}

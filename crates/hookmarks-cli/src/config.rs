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

impl Default for Config {
    fn default() -> Self {
        let config_dir = dirs::config_dir().expect("Could not determine config directory");
        let store_path = config_dir.join("hookmarks").join("store.db");

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
        let config_dir = dirs::config_dir().expect("Could not determine config directory");
        let config_path = config_dir.join("hookmarks").join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Config::default())
        }
    }

    /// Ensure config directory exists
    pub fn ensure_dir(&self) -> anyhow::Result<()> {
        let config_dir = dirs::config_dir().expect("Could not determine config directory");
        let hookmarks_dir = config_dir.join("hookmarks");

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

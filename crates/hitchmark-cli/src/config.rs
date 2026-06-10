//! Configuration file handling.
//!
//! Reads `~/.config/hitchmark/config.toml` and provides settings for the CLI.
//!
//! Environment variable overrides (useful for testing and CI):
//!   `HK_STORE_PATH` — override the SQLite store path
//!   `HK_CONFIG_DIR` — override the config directory

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

/// Resolve the hitchmark config directory, returning an error if HOME is not set.
fn hitchmark_config_dir() -> anyhow::Result<PathBuf> {
    // Allow override via env var (used in tests / CI)
    if let Ok(dir) = std::env::var("HK_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }
    dirs::config_dir()
        .map(|d| d.join("hitchmark"))
        .ok_or_else(|| anyhow::anyhow!(
            "Could not determine config directory. \
             Ensure $HOME (or $XDG_CONFIG_HOME) is set."
        ))
}

impl Default for Config {
    fn default() -> Self {
        // HK_STORE_PATH env var takes highest priority
        let store_path = if let Ok(p) = std::env::var("HK_STORE_PATH") {
            PathBuf::from(p)
        } else {
            dirs::config_dir()
                .map(|d| d.join("hitchmark").join("store.db"))
                .unwrap_or_else(|| PathBuf::from(".hitchmark/store.db"))
        };

        Config {
            store_path,
            auto_open: false,
            note_template: None,
        }
    }
}

impl Config {
    /// Load config from file, or return defaults if file doesn't exist.
    ///
    /// `HK_STORE_PATH` always overrides whatever is in the config file.
    pub fn load() -> anyhow::Result<Self> {
        let config_path = hitchmark_config_dir()?.join("config.toml");

        let mut cfg = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        // Env var overrides file setting (makes testing / CI reliable)
        if let Ok(p) = std::env::var("HK_STORE_PATH") {
            cfg.store_path = PathBuf::from(p);
        }

        Ok(cfg)
    }

    /// Ensure config directory and store parent directory exist.
    pub fn ensure_dir(&self) -> anyhow::Result<()> {
        let hitchmark_dir = hitchmark_config_dir()?;

        if !hitchmark_dir.exists() {
            std::fs::create_dir_all(&hitchmark_dir)?;
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

//! Tray preferences — stored in %APPDATA%\hitchmark\config.toml

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrayConfig {
    /// Port for hk serve (default: 2701)
    pub serve_port: u16,
    /// Explicit path to hk binary (empty = auto-detect)
    pub hk_path: String,
    /// Auto-start hk serve on tray launch
    pub auto_start_server: bool,
}

impl Default for TrayConfig {
    fn default() -> Self {
        TrayConfig {
            serve_port: 2701,
            hk_path: String::new(),
            auto_start_server: true,
        }
    }
}

impl TrayConfig {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(TrayConfig::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

fn config_path() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|d| d.join("hitchmark").join("tray.toml"))
        .ok_or_else(|| anyhow::anyhow!("Cannot determine %APPDATA% — is USERPROFILE set?"))
}

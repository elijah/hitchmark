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
    /// Auto-start hk watch on tray launch
    pub auto_start_watch: bool,
    /// Register this tray app to launch on Windows login
    pub auto_start_tray_on_login: bool,
}

impl Default for TrayConfig {
    fn default() -> Self {
        TrayConfig {
            serve_port: 2701,
            hk_path: String::new(),
            auto_start_server: true,
            auto_start_watch: false,
            auto_start_tray_on_login: true,
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

#[cfg(test)]
mod tests {
    use super::TrayConfig;

    #[test]
    fn default_enables_tray_login_start() {
        let cfg = TrayConfig::default();
        assert!(cfg.auto_start_tray_on_login);
        assert_eq!(cfg.serve_port, 2701);
    }

    #[test]
    fn serde_defaults_fill_missing_fields() {
        let cfg: TrayConfig = toml::from_str("serve_port = 2702").expect("valid toml");
        assert_eq!(cfg.serve_port, 2702);
        assert!(cfg.auto_start_server);
        assert!(!cfg.auto_start_watch);
        assert!(cfg.auto_start_tray_on_login);
    }
}

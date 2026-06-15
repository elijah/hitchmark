//! HTTP-first bridge to `hk serve`; subprocess fallback when server is not running.

use crate::config::TrayConfig;
use anyhow::Result;
use std::process::Command;

const SERVER_URL: &str = "http://127.0.0.1:2701";
#[cfg(target_os = "windows")]
const RUN_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
#[cfg(target_os = "windows")]
const TRAY_STARTUP_VALUE: &str = "HitchmarkTray";

/// Call GET /health — returns true if hk serve is running.
pub fn server_alive() -> bool {
    reqwest::blocking::get(format!("{SERVER_URL}/health"))
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Get the hook:// URI for a file path.
pub fn file_uri(path: &str, cfg: &TrayConfig) -> Result<String> {
    if server_alive() {
        let url = format!("{SERVER_URL}/uri?path={}", urlencoding::encode(path));
        let resp: serde_json::Value = reqwest::blocking::get(&url)?.json()?;
        if let Some(uri) = resp.get("uri").and_then(|v| v.as_str()) {
            return Ok(uri.to_string());
        }
        anyhow::bail!("Unexpected /uri response: {resp}");
    }
    run_hk(&["file", path], cfg)
}

/// List links for a URI (returns raw JSON string).
pub fn list_links_json(uri: &str, cfg: &TrayConfig) -> Result<String> {
    if server_alive() {
        let url = format!("{SERVER_URL}/links?uri={}", urlencoding::encode(uri));
        return Ok(reqwest::blocking::get(&url)?.text()?);
    }
    run_hk(&["list", uri, "--json"], cfg)
}

/// Open a hook:// URI.
pub fn open_uri(uri: &str, cfg: &TrayConfig) -> Result<()> {
    if server_alive() {
        let url = format!("{SERVER_URL}/open?uri={}", urlencoding::encode(uri));
        let resp = reqwest::blocking::get(&url)?;
        if !resp.status().is_success() {
            anyhow::bail!("open failed: HTTP {}", resp.status());
        }
        return Ok(());
    }
    run_hk(&["open", uri], cfg)?;
    Ok(())
}

/// Start `hk serve` as a detached background process.
pub fn start_server(cfg: &TrayConfig) -> Result<()> {
    let hk = find_hk_with_hint(cfg)?;
    Command::new(hk).arg("serve").spawn()?;
    Ok(())
}

/// Start `hk watch` as a detached background process.
pub fn start_watch(cfg: &TrayConfig) -> Result<()> {
    let hk = find_hk_with_hint(cfg)?;
    Command::new(hk).arg("watch").spawn()?;
    Ok(())
}

/// Ensure tray login startup registration matches the current config.
#[cfg(target_os = "windows")]
pub fn sync_tray_startup(cfg: &TrayConfig) -> Result<()> {
    use std::io::ErrorKind;
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = hkcu.create_subkey(RUN_KEY_PATH)?;

    if cfg.auto_start_tray_on_login {
        let exe = std::env::current_exe()?;
        run_key.set_value(TRAY_STARTUP_VALUE, &registry_command_for_path(&exe))?;
    } else if let Err(err) = run_key.delete_value(TRAY_STARTUP_VALUE) {
        if err.kind() != ErrorKind::NotFound {
            return Err(err.into());
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn sync_tray_startup(_cfg: &TrayConfig) -> Result<()> {
    Ok(())
}

/// Find the `hk` binary — checks PATH, then common install locations.
pub fn find_hk() -> Result<std::path::PathBuf> {
    find_hk_with_hint(&TrayConfig::default())
}

/// Find the `hk` binary with optional explicit path from tray config.
pub fn find_hk_with_hint(cfg: &TrayConfig) -> Result<std::path::PathBuf> {
    if !cfg.hk_path.trim().is_empty() {
        let p = std::path::PathBuf::from(cfg.hk_path.trim());
        if p.exists() {
            return Ok(p);
        }
        anyhow::bail!("Configured hk_path does not exist: {}", p.display());
    }

    // Check PATH first
    if let Ok(p) = which::which("hk") {
        return Ok(p);
    }
    // Common Windows install location (from WiX installer)
    let program_files =
        std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into());
    let candidate = std::path::PathBuf::from(program_files)
        .join("Hitchmark")
        .join("hk.exe");
    if candidate.exists() {
        return Ok(candidate);
    }
    // Cargo install location
    if let Some(home) = dirs::home_dir() {
        let cargo_bin = home.join(".cargo").join("bin").join("hk.exe");
        if cargo_bin.exists() {
            return Ok(cargo_bin);
        }
    }
    anyhow::bail!("hk binary not found. Install via: winget install hitchmark, or cargo install hitchmark-cli")
}

fn run_hk(args: &[&str], cfg: &TrayConfig) -> Result<String> {
    let hk = find_hk_with_hint(cfg)?;
    let output = Command::new(hk).args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("hk {}: {stderr}", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(target_os = "windows")]
fn registry_command_for_path(path: &std::path::Path) -> String {
    format!("\"{}\"", path.display())
}

// Minimal URL encoding without pulling in a heavy dep
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut out = String::with_capacity(s.len() * 3);
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char)
                }
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::urlencoding;

    #[test]
    fn percent_encodes_reserved_bytes() {
        assert_eq!(
            urlencoding::encode("C:\\Program Files\\hitch mark.txt"),
            "C%3A%5CProgram%20Files%5Chitch%20mark.txt"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn startup_registry_command_is_quoted() {
        let input = std::path::Path::new("C:\\Program Files\\Hitchmark\\hitchmark-tray.exe");
        assert_eq!(
            super::registry_command_for_path(input),
            "\"C:\\Program Files\\Hitchmark\\hitchmark-tray.exe\""
        );
    }
}

//! HTTP-first bridge to `hk serve`; subprocess fallback when server is not running.

use anyhow::Result;
use std::process::Command;

const SERVER_URL: &str = "http://127.0.0.1:2701";

/// Call GET /health — returns true if hk serve is running.
pub fn server_alive() -> bool {
    reqwest::blocking::get(format!("{SERVER_URL}/health"))
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Get the hook:// URI for a file path.
pub fn file_uri(path: &str) -> Result<String> {
    if server_alive() {
        let url = format!("{SERVER_URL}/uri?path={}", urlencoding::encode(path));
        let resp = reqwest::blocking::get(&url)?.text()?;
        return Ok(resp.trim().to_string());
    }
    run_hk(&["file", path])
}

/// List links for a URI (returns raw JSON string).
pub fn list_links_json(uri: &str) -> Result<String> {
    if server_alive() {
        let url = format!("{SERVER_URL}/links?uri={}", urlencoding::encode(uri));
        return Ok(reqwest::blocking::get(&url)?.text()?);
    }
    run_hk(&["list", uri, "--json"])
}

/// Open a hook:// URI.
pub fn open_uri(uri: &str) -> Result<()> {
    if server_alive() {
        let url = format!("{SERVER_URL}/open?uri={}", urlencoding::encode(uri));
        reqwest::blocking::get(&url)?;
        return Ok(());
    }
    run_hk(&["open", uri])?;
    Ok(())
}

/// Start `hk serve` as a detached background process.
pub fn start_server() -> Result<()> {
    let hk = find_hk()?;
    Command::new(hk).arg("serve").spawn()?;
    Ok(())
}

/// Find the `hk` binary — checks PATH, then common install locations.
pub fn find_hk() -> Result<std::path::PathBuf> {
    // Check PATH first
    if let Ok(p) = which::which("hk") {
        return Ok(p);
    }
    // Common Windows install location (from WiX installer)
    let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into());
    let candidate = std::path::PathBuf::from(program_files).join("Hitchmark").join("hk.exe");
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

fn run_hk(args: &[&str]) -> Result<String> {
    let hk = find_hk()?;
    let output = Command::new(hk).args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("hk {}: {stderr}", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Minimal URL encoding without pulling in a heavy dep
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut out = String::with_capacity(s.len() * 3);
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
                | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    }
}

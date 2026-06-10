//! Native Windows dialogs and OS-level helpers for the tray applet.
//!
//! - `input_box(prompt)` — shows a PowerShell InputBox (no extra deps)
//! - `foreground_app_path()` — Win32 GetForegroundWindow → process exe path
//! - `kill_server()` — taskkill /IM hk.exe /F
//! - `copy_to_clipboard(text)` — PowerShell Set-Clipboard

use anyhow::Result;
use std::process::Command;

/// Show a simple text-input dialog via PowerShell and return the entered string.
/// Returns `None` if the user cancels or the dialog cannot be shown.
pub fn input_box(title: &str, prompt: &str) -> Option<String> {
    // PowerShell one-liner: load VB runtime for InputBox
    let script = format!(
        r#"Add-Type -AssemblyName Microsoft.VisualBasic; \
[Microsoft.VisualBasic.Interaction]::InputBox('{prompt}', '{title}', '')"#,
        prompt = prompt.replace('\'', "''"),
        title = title.replace('\'', "''"),
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Get the file path of the process that owns the current foreground window.
/// Returns `None` on failure (e.g. if the foreground window is a system process).
#[cfg(target_os = "windows")]
pub fn foreground_app_path() -> Option<std::path::PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        )
        .ok()?;

        let mut buf = vec![0u16; 1024];
        let len = GetModuleFileNameExW(handle, None, &mut buf) as usize;
        if len == 0 {
            return None;
        }

        Some(std::path::PathBuf::from(OsString::from_wide(&buf[..len])))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn foreground_app_path() -> Option<std::path::PathBuf> {
    None
}

/// Stop `hk serve` by terminating all `hk.exe` processes.
/// Returns Ok(true) if any processes were killed, Ok(false) if none were running.
pub fn kill_server() -> Result<bool> {
    let output = Command::new("taskkill")
        .args(["/IM", "hk.exe", "/F"])
        .output()?;
    Ok(output.status.success())
}

/// Copy text to the Windows clipboard via PowerShell Set-Clipboard.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    // Escape single quotes for PowerShell string literal
    let escaped = text.replace('\'', "''");
    let script = format!("Set-Clipboard -Value '{escaped}'");
    let status = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Set-Clipboard failed (exit code: {:?})", status.code())
    }
}

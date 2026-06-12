//! Hitchmark Windows system tray applet.
//!
//! Runs as a background GUI process (no console window on Windows).
//! Shows a system tray icon with a context menu that mirrors the macOS menu bar app.
//!
//! Architecture:
//! - HTTP-first: calls `hk serve` REST API at http://127.0.0.1:2701
//! - Subprocess fallback: shells out to `hk` binary when serve is not running
//! - Preferences stored in `%APPDATA%\hitchmark\config.toml`
//!
//! Menu items:
//!   Copy URI for Foreground App   (Ctrl+Alt+H)
//!   ─────────────────────────────
//!   List Links...
//!   Open URI...
//!   ─────────────────────────────
//!   Start/Stop Server
//!   Open Dashboard
//!   ─────────────────────────────
//!   Preferences...
//!   About Hitchmark
//!   ─────────────────────────────
//!   Quit

// On Windows, hide the console window so only the tray icon appears.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod bridge;
mod config;
mod dialogs;
mod menu;

use anyhow::Result;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    // Load or create config
    let cfg = config::TrayConfig::load()?;

    if cfg.auto_start_server && !bridge::server_alive() {
        if let Err(e) = bridge::start_server(&cfg) {
            eprintln!("hitchmark-tray: failed to auto-start hk serve: {e}");
        }
    }
    if cfg.auto_start_watch {
        if let Err(e) = bridge::start_watch(&cfg) {
            eprintln!("hitchmark-tray: failed to auto-start hk watch: {e}");
        }
    }

    // Build tray menu
    let (tray_menu, ids) = menu::build_menu()?;

    // Create tray icon (embedded PNG, 16×16 or 32×32)
    let icon_bytes = include_bytes!("../assets/icon.png");
    let icon = load_icon(icon_bytes)?;

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Hitchmark")
        .with_icon(icon)
        .build()?;

    // Event loop — winit is required on Windows to keep the message pump running
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |_event, elwt| {
        // Drain menu events on each pump cycle
        while let Ok(event) = menu_channel.try_recv() {
            if let Err(e) = menu::handle_event(&event, &ids, &cfg) {
                eprintln!("hitchmark-tray: menu error: {e}");
            }
        }
        // Keep running until quit is selected (handle_event sets control_flow)
        let _ = elwt;
    })?;

    Ok(())
}

fn load_icon(bytes: &[u8]) -> Result<tray_icon::Icon> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to load tray icon: {e}"))?
        .into_rgba8();
    let (w, h) = img.dimensions();
    Ok(tray_icon::Icon::from_rgba(img.into_raw(), w, h)?)
}

//! Tray context menu construction and event dispatch.

use crate::{bridge, config::TrayConfig, dialogs};
use anyhow::Result;
use dirs;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};

/// IDs for menu items we need to match in event handling.
pub struct MenuIds {
    pub copy_uri: String,
    pub list_links: String,
    pub open_uri: String,
    pub toggle_server: String,
    pub open_dashboard: String,
    pub preferences: String,
    pub about: String,
    pub quit: String,
}

pub fn copy_uri_action(cfg: &TrayConfig) -> Result<()> {
    // Get the foreground application's exe path, convert to hook:// URI via hk file
    if let Some(app_path) = dialogs::foreground_app_path() {
        let path_str = app_path.to_string_lossy();
        match bridge::file_uri(&path_str, cfg) {
            Ok(uri) => {
                dialogs::copy_to_clipboard(&uri)
                    .unwrap_or_else(|e| eprintln!("[tray] clipboard error: {e}"));
                let _ = dialogs::show_balloon("Hitchmark", "Copied URI to clipboard");
                println!("[tray] copied URI: {uri}");
            }
            Err(e) => eprintln!("[tray] copy_uri: could not get URI for {path_str}: {e}"),
        }
    } else if let Some(path) = dialogs::input_box("Copy URI", "Enter file path:") {
        match bridge::file_uri(&path, cfg) {
            Ok(uri) => {
                dialogs::copy_to_clipboard(&uri)
                    .unwrap_or_else(|e| eprintln!("[tray] clipboard error: {e}"));
                let _ = dialogs::show_balloon("Hitchmark", "Copied URI to clipboard");
            }
            Err(e) => eprintln!("[tray] copy_uri: {e}"),
        }
    }

    Ok(())
}

pub fn build_menu() -> Result<(Menu, MenuIds)> {
    let server_label = if bridge::server_alive() {
        "Stop Server"
    } else {
        "Start Server"
    };

    let copy_uri_item = MenuItem::new("Copy URI for Foreground App\tCtrl+Alt+H", true, None);
    let sep1 = PredefinedMenuItem::separator();
    let list_links_item = MenuItem::new("List Links...", true, None);
    let open_uri_item = MenuItem::new("Open URI...", true, None);
    let sep2 = PredefinedMenuItem::separator();
    let toggle_server_item = MenuItem::new(server_label, true, None);
    let open_dashboard_item = MenuItem::new("Open Dashboard", true, None);
    let sep3 = PredefinedMenuItem::separator();
    let preferences_item = MenuItem::new("Preferences...", true, None);
    let about_item = MenuItem::new("About Hitchmark", true, None);
    let sep4 = PredefinedMenuItem::separator();
    let quit_item = MenuItem::new("Quit", true, None);

    let ids = MenuIds {
        copy_uri: copy_uri_item.id().0.clone(),
        list_links: list_links_item.id().0.clone(),
        open_uri: open_uri_item.id().0.clone(),
        toggle_server: toggle_server_item.id().0.clone(),
        open_dashboard: open_dashboard_item.id().0.clone(),
        preferences: preferences_item.id().0.clone(),
        about: about_item.id().0.clone(),
        quit: quit_item.id().0.clone(),
    };

    let menu = Menu::new();
    menu.append(&copy_uri_item)?;
    menu.append(&sep1)?;
    menu.append(&list_links_item)?;
    menu.append(&open_uri_item)?;
    menu.append(&sep2)?;
    menu.append(&toggle_server_item)?;
    menu.append(&open_dashboard_item)?;
    menu.append(&sep3)?;
    menu.append(&preferences_item)?;
    menu.append(&about_item)?;
    menu.append(&sep4)?;
    menu.append(&quit_item)?;

    Ok((menu, ids))
}

pub fn handle_event(event: &MenuEvent, ids: &MenuIds, cfg: &TrayConfig) -> Result<()> {
    let id = event.id().0.as_str();

    if id == ids.copy_uri {
        copy_uri_action(cfg)?;
    } else if id == ids.list_links {
        open::that("http://127.0.0.1:2701/#links").ok();
    } else if id == ids.open_uri {
        if let Some(uri) = dialogs::input_box("Open URI", "Enter hook:// URI:") {
            bridge::open_uri(&uri, cfg).unwrap_or_else(|e| eprintln!("[tray] open_uri: {e}"));
        }
    } else if id == ids.toggle_server {
        if bridge::server_alive() {
            match dialogs::kill_server() {
                Ok(true) => {
                    println!("[tray] hk serve stopped");
                    let _ = dialogs::show_balloon("Hitchmark", "hk serve stopped");
                }
                Ok(false) => println!("[tray] no hk.exe process found"),
                Err(e) => eprintln!("[tray] stop server: {e}"),
            }
        } else {
            bridge::start_server(cfg).unwrap_or_else(|e| eprintln!("[tray] start server: {e}"));
            let _ = dialogs::show_balloon("Hitchmark", "hk serve started");
        }
    } else if id == ids.open_dashboard {
        open::that("http://127.0.0.1:2701/").ok();
    } else if id == ids.preferences {
        // Open the config file in the default editor
        if let Some(config_dir) = dirs::config_dir() {
            let tray_toml = config_dir.join("hitchmark").join("tray.toml");
            // Ensure the file exists before opening
            if !tray_toml.exists() {
                if let Some(parent) = tray_toml.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&tray_toml, include_str!("../assets/default_tray.toml"));
            }
            open::that(&tray_toml).unwrap_or_else(|e| eprintln!("[tray] preferences: {e}"));
        }
    } else if id == ids.about {
        open::that("https://github.com/elijah/hitchmark").ok();
    } else if id == ids.quit {
        std::process::exit(0);
    }

    Ok(())
}

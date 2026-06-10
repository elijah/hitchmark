//! Tray context menu construction and event dispatch.

use crate::{bridge, config::TrayConfig, dialogs};
use anyhow::Result;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use dirs;

/// IDs for menu items we need to match in event handling.
pub struct MenuIds {
    pub copy_uri: u32,
    pub list_links: u32,
    pub open_uri: u32,
    pub toggle_server: u32,
    pub open_dashboard: u32,
    pub preferences: u32,
    pub about: u32,
    pub quit: u32,
}

pub fn build_menu() -> Result<(Menu, MenuIds)> {
    let server_label = if bridge::server_alive() { "Stop Server" } else { "Start Server" };

    let copy_uri_item      = MenuItem::new("Copy URI for Foreground App\tCtrl+Alt+H", true, None);
    let sep1               = PredefinedMenuItem::separator();
    let list_links_item    = MenuItem::new("List Links...", true, None);
    let open_uri_item      = MenuItem::new("Open URI...", true, None);
    let sep2               = PredefinedMenuItem::separator();
    let toggle_server_item = MenuItem::new(server_label, true, None);
    let open_dashboard_item = MenuItem::new("Open Dashboard", true, None);
    let sep3               = PredefinedMenuItem::separator();
    let preferences_item   = MenuItem::new("Preferences...", true, None);
    let about_item         = MenuItem::new("About Hitchmark", true, None);
    let sep4               = PredefinedMenuItem::separator();
    let quit_item          = MenuItem::new("Quit", true, None);

    let ids = MenuIds {
        copy_uri:       copy_uri_item.id().0,
        list_links:     list_links_item.id().0,
        open_uri:       open_uri_item.id().0,
        toggle_server:  toggle_server_item.id().0,
        open_dashboard: open_dashboard_item.id().0,
        preferences:    preferences_item.id().0,
        about:          about_item.id().0,
        quit:           quit_item.id().0,
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

pub fn handle_event(event: &MenuEvent, ids: &MenuIds, _cfg: &TrayConfig) -> Result<()> {
    let id = event.id().0;

    if id == ids.copy_uri {
        // Get the foreground application's exe path, convert to hook:// URI via hk file
        if let Some(app_path) = dialogs::foreground_app_path() {
            let path_str = app_path.to_string_lossy();
            match bridge::file_uri(&path_str) {
                Ok(uri) => {
                    dialogs::copy_to_clipboard(&uri)
                        .unwrap_or_else(|e| eprintln!("[tray] clipboard error: {e}"));
                    // Brief balloon notification would be ideal here; open::that is fallback
                    println!("[tray] copied URI: {uri}");
                }
                Err(e) => eprintln!("[tray] copy_uri: could not get URI for {path_str}: {e}"),
            }
        } else {
            // Fallback: prompt for path
            if let Some(path) = dialogs::input_box("Copy URI", "Enter file path:") {
                match bridge::file_uri(&path) {
                    Ok(uri) => {
                        dialogs::copy_to_clipboard(&uri)
                            .unwrap_or_else(|e| eprintln!("[tray] clipboard error: {e}"));
                    }
                    Err(e) => eprintln!("[tray] copy_uri: {e}"),
                }
            }
        }

    } else if id == ids.list_links {
        open::that("http://127.0.0.1:2701/#links").ok();

    } else if id == ids.open_uri {
        if let Some(uri) = dialogs::input_box("Open URI", "Enter hook:// URI:") {
            bridge::open_uri(&uri)
                .unwrap_or_else(|e| eprintln!("[tray] open_uri: {e}"));
        }

    } else if id == ids.toggle_server {
        if bridge::server_alive() {
            match dialogs::kill_server() {
                Ok(true)  => println!("[tray] hk serve stopped"),
                Ok(false) => println!("[tray] no hk.exe process found"),
                Err(e)    => eprintln!("[tray] stop server: {e}"),
            }
        } else {
            bridge::start_server()
                .unwrap_or_else(|e| eprintln!("[tray] start server: {e}"));
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
            open::that(&tray_toml)
                .unwrap_or_else(|e| eprintln!("[tray] preferences: {e}"));
        }

    } else if id == ids.about {
        open::that("https://github.com/elijah/hitchmark").ok();

    } else if id == ids.quit {
        std::process::exit(0);
    }

    Ok(())
}

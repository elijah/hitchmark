//! Tray context menu construction and event dispatch.

use crate::{bridge, config::TrayConfig};
use anyhow::Result;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};

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
    let copy_uri_item     = MenuItem::new("Copy URI for Foreground App\tCtrl+Alt+H", true, None);
    let sep1              = PredefinedMenuItem::separator();
    let list_links_item   = MenuItem::new("List Links...", true, None);
    let open_uri_item     = MenuItem::new("Open URI...", true, None);
    let sep2              = PredefinedMenuItem::separator();
    let toggle_server_item = MenuItem::new("Start Server", true, None);
    let open_dashboard_item = MenuItem::new("Open Dashboard", true, None);
    let sep3              = PredefinedMenuItem::separator();
    let preferences_item  = MenuItem::new("Preferences...", true, None);
    let about_item        = MenuItem::new("About Hitchmark", true, None);
    let sep4              = PredefinedMenuItem::separator();
    let quit_item         = MenuItem::new("Quit", true, None);

    let ids = MenuIds {
        copy_uri:      copy_uri_item.id().0,
        list_links:    list_links_item.id().0,
        open_uri:      open_uri_item.id().0,
        toggle_server: toggle_server_item.id().0,
        open_dashboard: open_dashboard_item.id().0,
        preferences:   preferences_item.id().0,
        about:         about_item.id().0,
        quit:          quit_item.id().0,
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
        // Get foreground window title via Windows API, then call hk file
        // For now: open a small dialog prompting for a path
        // TODO: integrate with win32 GetForegroundWindow → GetWindowText
        eprintln!("copy_uri: foreground app detection not yet implemented");

    } else if id == ids.list_links {
        // Open the web dashboard filtered to links view
        open::that("http://127.0.0.1:2701/#links").ok();

    } else if id == ids.open_uri {
        // TODO: show a small input dialog (win32 InputBox or custom winit window)
        eprintln!("open_uri: input dialog not yet implemented");

    } else if id == ids.toggle_server {
        if bridge::server_alive() {
            // Kill hk serve — for now, just inform user
            eprintln!("toggle_server: stop not yet implemented (kill process by name)");
        } else {
            bridge::start_server()?;
        }

    } else if id == ids.open_dashboard {
        open::that("http://127.0.0.1:2701/").ok();

    } else if id == ids.preferences {
        // TODO: open a preferences window (winit or native dialog)
        eprintln!("preferences: not yet implemented");

    } else if id == ids.about {
        open::that("https://github.com/elijah/hitchmark").ok();

    } else if id == ids.quit {
        std::process::exit(0);
    }

    Ok(())
}

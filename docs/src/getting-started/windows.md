# Windows Installation

## Install the CLI

### Option 1 — MSI installer (recommended)

Download `hitchmark-setup.msi` from the [latest release](https://github.com/elijah/hitchmark/releases/latest).

Run the installer — it places `hk.exe` in `C:\Program Files\Hitchmark\` and adds it to the system PATH. You do not need to restart your terminal if you open a new one.

Verify:
```powershell
hk --version
# hk 0.3.0
```

### Option 2 — winget

```powershell
winget install Hitchmark.Hitchmark
```

### Option 3 — Cargo

Requires [Rust](https://rustup.rs/) installed:

```powershell
cargo install hitchmark-cli
```

`hk.exe` is placed in `%USERPROFILE%\.cargo\bin\`, which is on PATH after `rustup` setup.

## System Tray App

The `hitchmark-tray.exe` applet gives quick access to Hitchmark from the notification area.

1. Download `hitchmark-tray.exe` from the [latest release](https://github.com/elijah/hitchmark/releases/latest) (or it is included in the MSI)
2. Run it — a Hitchmark icon appears in the system tray
3. Right-click the icon for the menu

To start automatically on login, place a shortcut in:
```
%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
```

## Configuration

Config file: `%APPDATA%\hitchmark\config.toml` (created automatically on first run)

Tray preferences: `%APPDATA%\hitchmark\tray.toml`

```toml
serve_port = 2701
hk_path = ""           # empty = auto-detect
auto_start_server = true
```

## Start the server

```powershell
hk serve
# 🔗 Hitchmark server listening on http://127.0.0.1:2701
```

Or let the tray app start it automatically when `auto_start_server = true`.

## Editor integrations

- **VS Code**: install the [Hitchmark extension](https://marketplace.visualstudio.com/items?itemName=hitchmark.hitchmark)
- **Neovim**: see [Neovim Plugin](./neovim.md)
- **Obsidian**: see [Obsidian Plugin](./obsidian.md)

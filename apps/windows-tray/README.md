# Hitchmark — Windows System Tray

Windows system tray applet for Hitchmark. Sits in the notification area and gives quick access to all Hitchmark features without opening a terminal.

## Features

- **Copy URI** — copy `hook://` URI for the foreground application (Ctrl+Alt+H)
- **List Links** — open the web dashboard filtered to the links view
- **Open URI** — prompt for a URI and open it
- **Start/Stop Server** — toggle `hk serve` in the background
- **Open Dashboard** — open the web dashboard at `http://127.0.0.1:2701`
- **Preferences** — edit `%APPDATA%\hitchmark\tray.toml`
- **Quit** — exit the tray

## Requirements

- Windows 10 or later
- `hk.exe` on `%PATH%` (installed via MSI or `cargo install hitchmark-cli`)

## Building

```powershell
# From repo root
cargo build --release -p hitchmark-tray
```

## Installing

The MSI installer (`apps/windows/hitchmark.wxs`) registers `hitchmark-tray.exe` as a startup item via the Windows registry. After installation, it appears in the notification area on next login.

Manual:
```powershell
# Add to startup
$startup = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
Copy-Item hitchmark-tray.exe $startup
```

## Configuration

`%APPDATA%\hitchmark\tray.toml`:
```toml
serve_port = 2701
hk_path = ""           # empty = auto-detect
auto_start_server = true
auto_start_watch = false
auto_start_tray_on_login = true
```

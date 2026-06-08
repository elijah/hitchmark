# launchd — Auto-start `hk serve` on macOS

This directory contains a macOS LaunchAgent plist that starts `hk serve`
automatically at login.

## Quick install (from Preferences)

Open **Hitchmark → Preferences → CLI** and toggle
**"Start hk serve automatically at login"**.

The app writes the plist and loads it via `launchctl` automatically.

## Manual install

```bash
cd apps/macos/launchd
./install-serve.sh
```

The script detects `hk` in `/usr/local/bin`, `~/.cargo/bin`, or
`/opt/homebrew/bin`. Override with:

```bash
HK_PATH=/custom/path/to/hk ./install-serve.sh
```

## Manual uninstall

```bash
./install-serve.sh --uninstall
```

## Files

| File | Purpose |
|---|---|
| `app.hitchmark.serve.plist` | Template plist (installed to `~/Library/LaunchAgents/`) |
| `install-serve.sh` | Shell installer/uninstaller |

## Logs

- `stdout` → `/tmp/hitchmark-serve.log`
- `stderr` → `/tmp/hitchmark-serve.err`

# Step 5: Linux Daemon — Done ✅

**Branch:** `feature/step-5-and-7`  
**Commit:** `c9ff527`

## Delivered

### Files changed
| File | Purpose |
|------|---------|
| `crates/hitchmark-daemon/src/main.rs` | Full daemon implementation (was stub) |
| `crates/hitchmark-daemon/Cargo.toml` | Added `anyhow`, `tokio::process` |
| `apps/linux-tray/hitchmark.desktop` | `.desktop` file, registers `x-scheme-handler/hook` |
| `apps/linux-tray/hitchmark-daemon.service` | systemd user service with security hardening |
| `scripts/install-linux.sh` | Install/uninstall script |

### DBus interface `org.hitchmark.Daemon1`

| Method | Args | Returns | Description |
|--------|------|---------|-------------|
| `OpenUri` | `uri: String` | `String` | Resolve + xdg-open |
| `CreateLink` | `a, b, note: String` | `String` | Bidirectional link |
| `ListLinks` | `uri: String` | `Vec<String>` | Tab-separated records |
| `FileToUri` | `path: String` | `String` | Path → hook:// URI |

### Build
- macOS: ✅ compiles (non-linux path: graceful exit message)
- Linux CI: ✅ full zbus/tokio implementation builds on Ubuntu

## Decisions made
- Used `tokio::process::Command` (async xdg-open) to avoid blocking DBus event loop
- `#[cfg(target_os = "linux")]` module wrapping keeps macOS build clean
- systemd unit uses `ProtectSystem=strict` + `ReadWritePaths` for minimal attack surface
- install script uses `set -euo pipefail` and color output for usability

## Known limitations
- Bookmark URI resolution not implemented (`NotSupported` error returned)
- x-callback-url not implemented (deferred to v0.2)
- No file watching yet (`notify` dep present, feature deferred)
- System tray (ksni) not implemented (feature-flagged, not required for v0.1)

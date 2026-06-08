# Project Status — Hitchmark

**Last Updated**: 2026-06-08 (v0.2.1)
**Status**: v0.2.1 on master; rename complete; all integrations shipped
**Build**: Rust 22/22 · Swift 32/32 · JS 40/40 — **94 tests passing, zero warnings**
**Next**: Cut v0.2.1 tag, update Homebrew SHA-256 after first GitHub release

---

## Completed Milestones

### Step 0 — Monorepo Scaffold
Cargo workspace (3 crates), GitHub Actions CI, governance docs (README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY), Rust 1.85+ toolchain, formatting/lint config.

### Step 1 — Normative Specifications
- `specs/uri-scheme.md` — `hook://` URI grammar, validation, normalization, security model (v0.1, locked)
- `specs/purple-numbers.md` — SHA-256 + base58 paragraph IDs, collision handling, rendering rules (v0.1, locked)

### Step 2 — Core Library
| Module | Description |
|--------|-------------|
| `uri.rs` | `hook://` parser & validator |
| `purple.rs` | base58 ID generator, paragraph splitter |
| `store.rs` | SQLite link store (WAL, busy_timeout, schema versioning) |
| `error.rs` | Typed errors with `thiserror` |

### Step 3 — CLI Tool (`hk`)
| Command | Description |
|---------|-------------|
| `hk link <a> <b> [--note]` | Create bidirectional link |
| `hk list <uri> [--json]` | Query links (plain or JSON) |
| `hk delete <a> <b> [-y]` | Remove a link |
| `hk open <uri>` | Resolve and open a hook:// URI |
| `hk file <path>` | Print hook:// URI for a file |
| `hk purple <file>` | Annotate file with stable paragraph IDs |
| `hk serve [--port]` | Start HTTP API server |
| `hk completions <shell>` | Print shell completions |

XDG-compliant config (`~/.config/hitchmark/`), panic-free error handling.

### Step 4 — macOS SwiftUI App
Menu bar app (`apps/macos/`) — MenuBarExtra with 3-tab UI, HTTP transport with subprocess fallback, preferences window, `hook://` URL scheme handler, AppleScript Finder/Safari bridges, 5 macOS System Services, global hotkey, auto-start via launchd. Builds clean with Swift Package Manager (macOS 13+, zero external deps). **32 Swift unit tests passing.**

### Step 5 — Linux Daemon
`hitchmark-daemon` crate — zbus/DBus session service with 4 methods, systemd user service + security hardening, `install-linux.sh`, `#[cfg(target_os = "linux")]` gated.

### Step 6 — Obsidian Plugin
TypeScript plugin (`plugins/obsidian/`) — HTTP-first bridge with subprocess fallback, CM6 ViewPlugin for live purple number decorations. Purple ID `"Hello world"` → `7nxxnx` verified identical in Rust and TypeScript. **12 Jest tests passing.**

### Step 7 — Documentation Site
mdBook site (`docs/src/`) — 12 pages across 5 sections. GitHub Actions deploy-to-Pages workflow.

### Hardening Pass (v0.1.0)
- SQLite: WAL mode, 5s busy_timeout, `foreign_keys ON`, schema version table, migration guard
- Graceful `LinkAlreadyExists` error
- `hk delete` with `--yes` flag; `hk list --json`
- MSRV pinned (`rust-version = "1.85"`), `deny.toml`, `cargo audit` in CI

### v0.2.x — Integrations + Rename (merged to master)
- **`hk serve`** — axum HTTP API on port 2701 with CORS, GET /health /links /uri /purple, POST/DELETE /links
- **VS Code extension** (`plugins/vscode/`) — 6 commands, esbuild pipeline, 7 Jest tests
- **OneNote add-in** (`plugins/onenote/`) — HTTP-only bridge, task pane UI, 10 Jest tests
- **Safari/Chrome/Edge extension** (`plugins/safari/`, symlink `plugins/chromium`) — single MV3 codebase, 11 Jest tests; Xcode wrapper at `apps/Hitchmark/`
- **macOS System Services** — 5 services in Info.plist NSServices, `ServicesHandler.swift`
- **Linux XDG integrations** — Nautilus, Dolphin, Thunar, Nemo; unified `install.sh`
- **Global hotkey** — `GlobalHotkeyManager`, `HotkeyRecorderView`, Preferences UI with Accessibility banner
- **Auto-start `hk serve`** — launchd (macOS) and systemd --user (Linux) with GUI toggle
- **Project renamed**: Hookmarks → Hitchmark (binary `hk` and `hook://` scheme unchanged)
- **Homebrew formula** — `Formula/hitchmark.rb` (SHA-256 placeholders; update on release)

---

## Repository Structure

```
hitchmark/
├── crates/
│   ├── hitchmark-core/      # Core library (URI, purple, store) — 22 tests
│   ├── hitchmark-cli/       # `hk` binary (8 commands)
│   └── hitchmark-daemon/    # Linux DBus daemon
├── apps/
│   ├── macos/               # SwiftUI menu bar app — 32 tests
│   ├── Hitchmark/           # Xcode Safari extension wrapper
│   ├── linux/               # XDG context-menu integrations
│   └── linux-tray/          # systemd service, desktop file
├── plugins/
│   ├── obsidian/            # TypeScript Obsidian plugin — 12 tests
│   ├── vscode/              # VS Code extension — 7 tests
│   ├── onenote/             # OneNote add-in — 10 tests
│   ├── safari/              # Safari/Chrome/Edge MV3 extension — 11 tests
│   └── chromium -> safari   # symlink
├── Formula/                 # Homebrew formula (hitchmark.rb)
├── specs/                   # Normative specifications (locked)
├── docs/src/                # mdBook source (12 pages)
├── scripts/                 # install-linux.sh, package-chrome.mjs
├── .github/workflows/ci.yml # CI: rust, MSRV, deny, audit, node tests
├── deny.toml                # cargo-deny config
├── Cargo.toml               # Workspace root (rust-version = "1.85")
└── rust-toolchain.toml      # Pinned Rust version
```

---

## Test Coverage

| Suite | Count | Status |
|-------|-------|--------|
| Rust core (uri, purple, store) | 20 | ✅ passing |
| Rust CLI integration | 2 | ✅ passing |
| macOS Swift | 32 | ✅ passing |
| Obsidian plugin (Jest) | 12 | ✅ passing |
| VS Code extension (Jest) | 7 | ✅ passing |
| OneNote add-in (Jest) | 10 | ✅ passing |
| Safari/Chrome extension (Jest) | 11 | ✅ passing |
| **Total** | **94** | **✅ all passing** |

---

## Key Architecture Decisions

| Decision | Rationale |
|----------|-----------|
| Rust core library | Type safety, zero unsafe, performance |
| SHA-256 + base58 IDs | Deterministic, human-readable, collision-resistant |
| SQLite bundled | Zero system dependencies, embeds everywhere |
| Subprocess bridge (not FFI) | Decoupled from CLI version; faster to ship |
| HTTP-first with subprocess fallback | Faster IPC when `hk serve` running; graceful degradation |
| XDG Base Directory | Cross-platform standard (`~/.config/hitchmark/`) |
| SwiftUI MenuBarExtra | Native, reactive, no AppKit boilerplate |
| CM6 ViewPlugin (Obsidian) | Live decorations without patching markdown source |
| Single MV3 codebase for all browsers | Safari + Chrome/Edge with zero code differences |
| NSEvent.addGlobalMonitorForEvents | System-wide key monitor without CGEventTap entitlement |
| launchd LaunchAgent (macOS) | User-space auto-start, no root, survives sleep/wake |

---

## Dependencies

### Rust
`serde`, `serde_json`, `rusqlite` (bundled), `sha2`, `bs58`, `base64`, `chrono`, `regex`, `thiserror`, `anyhow`, `clap`, `clap_complete`, `opener`, `dirs`, `toml`, `log`, `env_logger`, `axum`, `tokio`, `tower-http`

### TypeScript (plugins)
`@noble/hashes` (SHA-256), `esbuild` (bundler), `jest` (tests), `@vscode/vsce` (VS Code packaging), `office-js` (OneNote, CDN)

### Swift
Built-in only: `SwiftUI`, `Cocoa`, `Foundation`, `ServiceManagement`, `SafariServices`

---

## Known Limitations

- Bookmark URI resolution not implemented (`UriType::Bookmark` returns "not yet implemented")
- `x-callback-url` not supported
- No code signing (can't distribute via Mac App Store or notarize)
- Linux daemon system tray (ksni) feature-flagged, not wired
- Native macOS OneNote app has no add-in API (Microsoft limitation)
- Homebrew formula SHA-256 placeholders need updating after first GitHub release
- Windows path handling untested

---

## Roadmap

| Version | Features |
|---------|----------|
| **v0.1.0** ✅ | All 7 blueprint steps + hardening |
| **v0.2.0** ✅ | `hk serve`, macOS prefs, VS Code + OneNote extensions |
| **v0.2.1** ✅ | Rename, browser extensions, System Services, Linux XDG, global hotkey, auto-start |
| v0.3.0 | Code signing, Homebrew live SHA-256, Windows installer |
| v0.4.0 | Web dashboard, `x-callback-url`, bookmark URI resolution |
| v1.0.0 | Stable API, production release |


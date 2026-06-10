# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.4.0] — 2026-06-10

Cleanup sprint: legacy file removal, live Homebrew SHA-256, Windows cross-compile + installer, Windows system tray, code signing documentation.

### Added

#### Windows
- `apps/windows-tray/` — Rust system tray applet (`hitchmark-tray`)
  - `tray-icon` + `winit` event loop; `windows_subsystem = "windows"` (no console)
  - HTTP-first bridge to `hk serve`; `find_hk()` checks PATH, Program Files, `~/.cargo/bin`
  - Context menu: Copy URI, List Links, Open URI, Start/Stop Server, Open Dashboard, Preferences, About, Quit
  - Config at `%APPDATA%\hitchmark\tray.toml`
- `apps/windows/hitchmark.wxs` — WiX v4 installer stub: installs `hk.exe`, adds to system PATH, uninstaller in Add/Remove Programs
- `.github/workflows/release.yml` — cross-platform release CI: macOS universal binary (arm64+x86_64 lipo), Linux x86_64 tar.gz, Windows x86_64 zip

#### Documentation
- `docs/src/project/codesigning-macos.md` — full macOS notarization guide (Developer ID cert, codesign hardened runtime, notarytool, DMG staple, entitlements)
- `docs/src/project/codesigning-windows.md` — Windows Authenticode guide (signtool, PFX import, MSI signing, winget manifest)
- `.github/workflows/release-macos.yml` — notarized macOS release stub (manual trigger; needs Apple secrets)
- `.github/workflows/release-windows.yml` — Authenticode Windows release stub (manual trigger; needs cert secrets)

### Fixed
- `crates/hitchmark-cli/src/commands/watch.rs` — handle Linux inotify `RenameMode::From`/`To` split rename events via thread-local pending-rename tracking (previously only `RenameMode::Both` was handled, so Linux renames were silently missed)
- `crates/hitchmark-cli/Cargo.toml` — removed `macos_fsevent` feature from `notify` (v6 selects the right backend per-platform automatically; the feature flag caused cross-compile failures on Windows/Linux)
- `crates/hitchmark-cli/src/path.rs` — platform-aware `normalize_dots` test; simplified `expand_path`

### Changed
- `Formula/hitchmark.rb` — updated `url` and `sha256` to v0.3.0 tarball (live)
- Workspace version bumped to `0.3.0`

### Removed
- `apps/linux-tray/hookmarks-daemon.service` — stale legacy file superseded by `hitchmark-serve.service`

---

## [0.3.0] — 2026-06-09

Enhancement sprint: garbage collection, backup/restore, x-callback-url, file watching, web dashboard, and Neovim plugin.

### Added

#### CLI
- **`hk gc`** — scan and optionally delete stale links & bookmarks (files that no longer exist); `--dry-run` by default, `--delete` to act; `--json` output; exit code 1 when stale entries found (script-friendly)
- **`hk export`** — export all links and bookmarks to NDJSON (default) or `--format json`; `--only links|bookmarks`; `--out FILE`
- **`hk import`** — import from NDJSON/JSON; validates all records before writing; idempotent (skips duplicates); `--dry-run` flag
- **`hk watch`** — watch parent directories of bookmarked files; automatically calls `update_bookmark_path` on rename; warns on delete (suggests `hk gc`)
- **`x-callback-url` support** — `hk open` now fully handles `x-success`/`x-error` callbacks for actions `create-link`, `open`, `copy-uri`; percent-encode/decode utilities added to core

#### Core (`hitchmark-core`)
- `scan_stale_links()` / `scan_stale_bookmarks()` — return IDs of entries whose file paths no longer exist
- `delete_links_involving()` / `delete_bookmarks_by_ids()` — bulk delete helpers
- `list_all_links()` — full link scan ordered by `created_at` (used by export and dashboard)
- `import_bookmark(id, path)` — fixed-UUID insert; returns `BookmarkAlreadyExists` on duplicate
- `BookmarkAlreadyExists` error variant
- `XCallbackUri { action, params }` struct replacing bare string in `UriType::XCallbackUrl`
- `parse_query_string()`, `percent_decode()`, `percent_encode()` in `uri.rs`

#### `hk serve` — Web Dashboard
- `GET /` — serves embedded single-page web dashboard (no build step, no external deps)
- `GET /links/all` — returns all links as JSON array
- `GET /bookmarks` — returns all bookmarks as JSON array
- Dashboard features: links + bookmarks tables, per-section search, sortable columns, copy-URI buttons, 30s auto-refresh, dark/light mode (`prefers-color-scheme`), `prefers-reduced-motion`, responsive layout

#### Neovim Plugin (`plugins/neovim/`)
- `bridge.lua` — HTTP-first transport (curl) with `hk` subprocess fallback; `find_hk()` checks PATH and common install locations
- `commands.lua` — `:HkFile` (copy URI to `+`), `:HkLink` (prompt + link), `:HkList` (quickfix), `:HkPurple` (extmark virtual text), `:HkOpen` (URI under cursor or prompted)
- `init.lua` — `setup(opts)` with keymap registration; configurable `serve_url`, `hk_path`, keymaps
- `plugin/hitchmark.vim` — autoload guard shim
- `tests/bridge_spec.lua` — busted specs for all bridge entry points
- `README.md` — lazy.nvim/packer/vim-plug install, command reference, config options

#### Platform
- `apps/macos/launchd/app.hitchmark.watch.plist` — LaunchAgent for `hk watch` (auto-start on login)
- `apps/linux-tray/hitchmark-watch.service` — systemd `--user` unit for `hk watch`

### Changed
- `UriType::XCallbackUrl` now carries `XCallbackUri` struct (breaking change in core — update any pattern matches)

---

## [0.2.1] — 2026-06-08

Project-wide rename from Hookmarks → Hitchmark, global hotkey, auto-start, and platform integrations.

### Added

#### macOS App
- **Global hotkey** — `GlobalHotkeyManager` registers `NSEvent.addGlobalMonitorForEvents`; fires `copyURIForFrontApp()` via AppleScript; configurable from Preferences → Hotkeys with live key recorder and Accessibility permission banner
- **Auto-start `hk serve`** — `ServeAgent.swift` manages a launchd `LaunchAgent` plist; toggle in Preferences → CLI; `apps/macos/launchd/` has shell installer/uninstaller
- 5 macOS System Services: Copy URI, Link Files, Show Links, Open URI, Convert Path to URI — registered in `NSServices` (Info.plist) and wired via `ServicesHandler`

#### Browser Extensions
- Safari, Chrome, Edge, all Chromium browsers — single MV3 codebase (`plugins/safari/`); `contextMenus` permission added; background changed to IIFE format for Safari compatibility
- `plugins/chromium → safari` symlink for discoverability
- `scripts/package-chrome.mjs` — produces clean Web Store zip
- Xcode wrapper at `apps/Hitchmark/` (generated by `xcrun safari-web-extension-converter`)

#### Linux Integrations
- XDG context-menu actions for GNOME Nautilus, KDE Dolphin, Xfce Thunar, Cinnamon Nemo
- `apps/linux/install.sh` — unified installer with auto DE-detection, `--dry-run`, `--uninstall`
- systemd `--user` service: `apps/linux-tray/hitchmark-serve.service` with shell installer

#### Infrastructure
- `Formula/hitchmark.rb` — Homebrew formula (SHA-256 placeholders; update after first GitHub release)
- Config dir migrated: `~/.config/hookmarks/` → `~/.config/hitchmark/`
- DB default: `.hookmarks/store.db` → `.hitchmark/store.db`

### Changed

- **Project renamed**: Hookmarks → Hitchmark throughout (binary `hk` and `hook://` URI scheme unchanged)
- `@objc` service selectors: `hookmarks*` → `hitchmark*` (matched in Info.plist `NSMessage`)
- Crate names: `hookmarks-{core,cli,daemon}` → `hitchmark-{core,cli,daemon}`
- Bundle ID: `com.elw.hookmarks` → `app.hitchmark`
- DBus name: `org.not_hookmarks.Daemon` → `org.hitchmark.Daemon`
- Homebrew formula: `hookmarks.rb` → `hitchmark.rb`
- Linux tray desktop: `not-hookmarks.desktop` → `hitchmark.desktop`
- Stale `crates/hookmarks-*` and `apps/macos/Sources/Hookmarks/` directories removed

### Fixed
- Safari extension: removed `"type": "module"` from background config (Safari MV3 incompatibility)
- macOS Services: curly-quote compile error in string literal
- `contextMenus` permission missing from extension manifest

---

## [0.2.0] — 2026-06-08

### Added

#### CLI
- `hk serve` — local axum HTTP API server (GET /health, /links, /uri, /purple; POST /links; DELETE /links)
- `hk delete <a> <b> [-y]` — remove a bidirectional link
- `hk list --json` — machine-readable JSON output
- `hk completions <shell>` — generate bash/zsh/fish/elvish/powershell completions

#### macOS App
- All preferences now persisted via `@AppStorage` (UserDefaults)
- Launch at login via `SMAppService` (macOS 13+)
- Icon style picker wired (was hardcoded)
- CLI tab: auto-detect path shown, server URL field with live probe button
- HTTP transport in `HKBridge` — uses `hk serve` when available, falls back to subprocess

#### Obsidian Plugin
- Bridge updated to use `hk list --json` (was fragile tab-parsing)
- HTTP transport added — uses `hk serve` when `serverUrl` is set

#### VS Code Extension (`plugins/vscode/`) — new
- 6 commands: Copy URI, Copy URI with paragraph ID, List Links, Open URI, Show Purple Numbers, Start Server
- Context menus on editor and Explorer
- `Ctrl+Alt+H` keybinding for copy URI
- HTTP-first bridge with subprocess fallback
- 7 Jest tests

#### OneNote Add-in (`plugins/onenote/`) — new
- Office Add-in (TypeScript, Office JS API)
- Platforms: OneNote Online (macOS + Windows), OneNote for Windows desktop
- Task pane UI: current page URI, link list, create/delete links
- Ribbon button: one-click copy of page hook:// URI
- `HKAddInBridge.buildPageUri()` — URL-safe base64 encoding of OneNote page URLs
- Requires `hk serve` (browser sandbox cannot spawn subprocesses)
- 10 Jest tests

#### Infrastructure
- `deny.toml` — license allow-list, CVE deny, wildcard dep ban
- CI expanded: MSRV (1.85), `cargo deny`, `cargo audit`, Node.js tests
- MSRV pinned: `rust-version = "1.85"` in workspace `Cargo.toml`

#### Tests
- 24 Swift unit tests (HKBridge path resolution, prefs key contracts, error display)
- 7 VS Code bridge tests
- 10 OneNote bridge tests

### Fixed
- SQLite WAL mode, 5s busy_timeout, `foreign_keys ON`
- Graceful `LinkAlreadyExists` error (no raw DB error surfaced to users)
- Schema version table with migration guard
- All `expect()` panics in `config.rs` replaced with `anyhow::Result`
- `HKBridge.locateHK()` now checks user `cliPath` pref before auto-detecting

---

## [0.1.0] — 2026-06-07

First complete release covering all 7 blueprint steps plus a stability hardening pass.

### Added

#### Core (`hitchmark-core`)
- `hook://` URI parser and validator (`uri.rs`) with base64url encoding for file paths
- Purple number generator (`purple.rs`) — SHA-256 → base58 stable paragraph IDs, first 6 chars, extends to 8 on collision
- SQLite link store (`store.rs`) — bidirectional links, WAL mode, 5s busy_timeout, `foreign_keys ON`
- Schema version table with migration guard (`SCHEMA_VERSION = 1`)
- Typed error enum with `thiserror` — includes `LinkAlreadyExists` and `SchemaTooNew` variants

#### CLI (`hk`)
- `hk link <a> <b> [--note]` — create bidirectional link; friendly message on duplicate
- `hk list <uri> [--json]` — list links in plain text or structured JSON
- `hk delete <a> <b> [-y]` — remove a link with interactive confirmation
- `hk open <uri>` — resolve and open a `hook://` URI
- `hk file <path>` — convert file path to `hook://` URI
- `hk purple <file> [--format markdown|json]` — annotate file with stable paragraph IDs
- XDG-compliant config and store at `~/.config/hookmarks/`
- Panic-free error handling throughout

#### macOS App (`apps/macos`)
- SwiftUI menu bar app using `MenuBarExtra` (macOS 13+)
- 3-tab interface: Link, List, Finder
- Subprocess bridge to `hk` binary (`HKBridge.swift`)
- AppleScript bridges for Finder and Safari (`FinderBridge.swift`)
- Preferences window with 4 tabs: General, Hotkeys, CLI, About
- `hook://` URL scheme handler (`AppDelegate.swift`)
- Zero external Swift dependencies

#### Linux Daemon (`hitchmark-daemon`)
- zbus/DBus session service with 4 methods: `OpenUri`, `CreateLink`, `ListLinks`, `FileToUri`
- systemd user service with security hardening directives
- `install-linux.sh` installer script
- `#[cfg(target_os = "linux")]` gating — compiles clean on macOS

#### Obsidian Plugin (`plugins/obsidian`)
- CM6 `ViewPlugin` for live purple-number gutter decorations
- `ItemView` link panel sidebar (5 registered commands)
- Subprocess bridge to `hk --json` for structured data
- SHA-256 + base58 implementation cross-compatible with Rust (`"Hello world"` → `7nxxnx`)
- 12 Jest unit tests

#### Documentation (`docs/`)
- mdBook site with 12 pages across 5 sections
- GitHub Actions deploy-to-Pages workflow

#### Specifications (`specs/`)
- `specs/uri-scheme.md` — normative `hook://` URI spec v0.1 (locked)
- `specs/purple-numbers.md` — normative purple numbers spec v0.1 (locked)

#### Infrastructure
- Cargo workspace with 3 crates, MSRV `rust-version = "1.85"`
- `deny.toml` — license allow-list (MIT/Apache), vulnerability deny, wildcard ban
- CI: build, test, fmt, clippy, MSRV check, `cargo deny`, `cargo audit`, Node.js plugin tests

### Known Limitations
- Bookmark URI resolution not yet implemented
- `x-callback-url` not supported
- macOS app preferences not persisted across restarts
- No global hotkey support
- No code signing

[0.1.0]: https://github.com/elijah/hitchmark/releases/tag/v0.1.0
[0.2.0]: https://github.com/elijah/hitchmark/releases/tag/v0.2.0
[0.2.1]: https://github.com/elijah/hitchmark/releases/tag/v0.2.1
[0.3.0]: https://github.com/elijah/hitchmark/releases/tag/v0.3.0
[0.4.0]: https://github.com/elijah/hitchmark/releases/tag/v0.4.0

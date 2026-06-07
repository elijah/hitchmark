# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] — 2026-06-07

First complete release covering all 7 blueprint steps plus a stability hardening pass.

### Added

#### Core (`hookmarks-core`)
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

#### Linux Daemon (`hookmarks-daemon`)
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
- Cargo workspace with 3 crates, MSRV `rust-version = "1.75"`
- `deny.toml` — license allow-list (MIT/Apache), vulnerability deny, wildcard ban
- CI: build, test, fmt, clippy, MSRV check, `cargo deny`, `cargo audit`, Node.js plugin tests

### Known Limitations
- Bookmark URI resolution not yet implemented
- `x-callback-url` not supported
- macOS app preferences not persisted across restarts
- No global hotkey support
- No code signing

[0.1.0]: https://github.com/yourusername/not-hookmarks/releases/tag/v0.1.0

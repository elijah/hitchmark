# Changelog

All notable changes to Hitchmark are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

The full changelog with commit links is in [CHANGELOG.md](https://github.com/elijah/hitchmark/blob/master/CHANGELOG.md).

---

## [0.5.0] — 2026-06-11

Quality sprint: CLI integration tests, crate publish metadata, spec refresh, HTTP API expansion.

### Added
- **19 CLI integration tests** — full coverage of `hk file`, `hk link`, `hk list`, `hk delete`, `hk gc`, `hk export`, `hk import`, `hk purple`, `hk completions`
- `HK_STORE_PATH` / `HK_CONFIG_DIR` env var overrides for test isolation
- `hk version [--verbose]` subcommand alias
- `GET /open?uri=` HTTP endpoint — open any `hook://` URI via the OS
- `cargo publish` metadata in both crates (ready for crates.io)
- Per-crate `README.md` files
- Real 32×32 PNG tray icon (Windows)

### Changed
- `hk gc --delete` exits 0 on successful cleanup
- `hk file` errors on nonexistent paths
- Specs updated: renamed, dates corrected, CLI flags fixed

---

## [0.4.0] — 2026-06-10

Cleanup sprint: legacy removal, live Homebrew SHA-256, Windows port + installer, Windows system tray, code-signing docs.

### Added
- **Windows system tray** (`apps/windows-tray/`) — full context menu, HTTP bridge, config, preferences
- **Windows installer stub** (`apps/windows/hitchmark.wxs`) — WiX v4
- **Cross-platform release CI** (`.github/workflows/release.yml`) — macOS universal, Linux, Windows
- Code-signing guides for macOS (notarization) and Windows (Authenticode + winget)
- Docs: Windows install guide, Neovim plugin guide, web dashboard reference

### Changed
- Homebrew formula: live SHA-256 for v0.3.0 tarball
- Linux `hk watch`: fixed inotify split-rename bug via `PENDING_RENAME` thread-local
- `PROJECT_STATUS.md` updated to v0.4.0

---

## [0.3.0] — 2026-06-09

### Added
- `hk gc` — garbage-collect stale links (dry-run + `--delete`)
- `hk export` / `hk import` — NDJSON roundtrip
- `hk watch` — file rename auto-repair (macOS FSEvents, Linux inotify, Windows RDCW)
- `hk serve` — local HTTP API (port 2701) with web dashboard
- `hk bookmark` — stable UUID-based file references
- Obsidian plugin: purple number rendering, link panel, 5 commands

---

## [0.2.0] — 2026-06-08

### Added
- Homebrew formula (`Formula/hitchmark.rb`)
- Shell completions: bash, zsh, fish, PowerShell
- `hk manpage` — generate and install hk(1)
- macOS app: improved menu bar UI, Finder integration

---

## [0.1.0] — 2026-06-06

### Added
- Cargo workspace: `hitchmark-core`, `hitchmark-cli`
- `hook://` URI scheme v0.1 (file, bookmark, x-callback-url)
- `PurpleNumberGenerator` — SHA-256 → base58, collision detection
- `LinkStore` — SQLite bidirectional link store
- `hk link`, `hk list`, `hk open`, `hk file`, `hk purple`
- macOS SwiftUI menu bar app with `hook://` URL scheme handler
- Linux DBus daemon + systemd user service
- Obsidian plugin scaffold

---

## Algorithm Compatibility

The purple number algorithm produces identical output in all implementations:

| Implementation | Location | Function |
|---|---|---|
| Rust | `hitchmark-core` | `PurpleNumberGenerator::generate()` |
| TypeScript | `plugins/obsidian` | `generatePurpleId()` |

Test vector: `"Hello world"` → `7nxxnx`

# Changelog

All notable changes to Hookmarks are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added
- Step 7: mdBook documentation site (this site)

---

## [0.1.0] — 2026-06-06

### Added

**Step 0: Monorepo scaffold**
- Cargo workspace with three crates
- CI/CD pipeline (GitHub Actions)
- Governance docs: README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY

**Step 1: Normative specifications**
- `specs/uri-scheme.md` — `hook://` URI scheme v0.1
- `specs/purple-numbers.md` — purple number algorithm v0.1

**Step 2: hitchmark-core**
- `HookUri` parser and serializer
- `PurpleNumberGenerator` (SHA-256 → base58, collision detection)
- `LinkStore` — SQLite-backed bidirectional link storage
- 8 unit tests, zero unsafe code

**Step 3: hitchmark-cli (hk)**
- `hk link` — create bidirectional links
- `hk list` — query links for a resource
- `hk open` — resolve and open hook:// URIs
- `hk file` — convert file path to hook:// URI
- `hk purple` — generate purple numbers (markdown + JSON output)
- XDG-compliant config (~/.config/hookmarks/)

**Step 4: macOS SwiftUI app**
- Menu bar icon (MenuBarExtra) with 3-tab interface
- Link tab: create bidirectional links
- List tab: query existing links
- Finder tab: get hook:// URI for selected Finder file
- AppDelegate: hook:// URL scheme handler
- HKBridge: subprocess integration to hk CLI
- FinderBridge: AppleScript integration
- Preferences window (4 tabs)
- Info.plist with hook:// URL scheme registration

**Step 5: Linux daemon**
- DBus session service: org.hitchmark.Daemon
- Interface org.hitchmark.Daemon1: OpenUri, CreateLink, ListLinks, FileToUri
- .desktop file with x-scheme-handler/hook
- systemd user service with security hardening
- install-linux.sh with --uninstall support

**Step 6: Obsidian community plugin**
- Purple numbers in live editor (CodeMirror 6 ViewPlugin)
- §id click-to-copy-URI annotations
- Link panel sidebar (ItemView)
- 5 commands (copy note URI, copy paragraph URI, create link, open panel, refresh)
- HKBridge subprocess integration
- Settings tab with color picker, CLI path config, test button
- 12 unit tests (purple algorithm verified byte-compatible with Rust)
- Builds to 35KB main.js bundle

---

## Algorithm compatibility

The purple number algorithm is implemented identically in:
- **Rust** (`hitchmark-core`) — `PurpleNumberGenerator::generate()`
- **TypeScript** (`plugins/obsidian`) — `generatePurpleId()`

Test vector: `"Hello world"` → `7nxxnx` (both implementations)

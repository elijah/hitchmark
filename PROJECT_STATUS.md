# Project Status — Hookmarks

**Last Updated**: v0.2.0-dev (extension build pipelines complete)
**Status**: v0.1.0 shipped; v0.2.0 feature work in progress
**Build**: All 75 tests passing, zero warnings
**Branch**: `master` (feature/extension-completions pending merge)

---

## Completed Milestones

### Step 0 — Monorepo Scaffold
Cargo workspace (3 crates), GitHub Actions CI, governance docs (README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY), Rust 1.75+ toolchain, formatting/lint config.

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
| `hk uri <file>` | Print hook:// URI for a file |
| `hk purple <file>` | Annotate file with stable paragraph IDs |
| `hk serve [--port]` | Start HTTP API server |
| `hk completions <shell>` | Print shell completions |

XDG-compliant config (`~/.config/hookmarks/`), panic-free error handling.

### Step 4 — macOS SwiftUI App
Menu bar app (`apps/macos/`) — MenuBarExtra with 3-tab UI (Link / List / Finder), HTTP transport with subprocess fallback, preferences window (CLI path, server URL, hotkey, launch-at-login via SMAppService), `hook://` URL scheme handler, AppleScript Finder/Safari bridges. Builds clean with Swift Package Manager (macOS 13+, zero external deps). **24 Swift unit tests passing.**

### Step 5 — Linux Daemon
`hitchmark-daemon` crate — zbus/DBus session service with 4 methods (`OpenUri`, `CreateLink`, `ListLinks`, `FileToUri`), systemd user service + security hardening, `install-linux.sh`, `#[cfg(target_os = "linux")]` gated.

### Step 6 — Obsidian Plugin
TypeScript plugin (`plugins/obsidian/`) — HTTP-first bridge with subprocess fallback, CM6 ViewPlugin for live purple number decorations. Purple ID `"Hello world"` → `7nxxnx` verified identical in Rust and TypeScript. **12 Jest tests passing.**

### Step 7 — Documentation Site
mdBook site (`docs/src/`) — 12 pages across 5 sections (Getting Started, URI Scheme, Purple Numbers, CLI Reference, Integration). GitHub Actions deploy-to-Pages workflow.

### Hardening Pass (v0.1.0)
- SQLite: WAL mode, 5s busy_timeout, `foreign_keys ON`, schema version table, migration guard
- Graceful `LinkAlreadyExists` error
- `hk delete` with `--yes` flag; `hk list --json`
- 9 new URI edge-case tests
- MSRV pinned (`rust-version = "1.75"`), `deny.toml`, `cargo audit` in CI

### v0.2 Features (merged to master)
- **`hk serve`** — axum HTTP API on port 2701: `GET /health`, `GET /links`, `POST /links`, `DELETE /links`, `GET /uri`, `GET /purple`. CORS restricted to local origins.
- **macOS prefs persistence** — all prefs via `@AppStorage`; `cliPath` UserDefaults used by bridge; launch-at-login via `SMAppService`.
- **Homebrew formula** — `Formula/hookmarks.rb` with `brew test` block; SHA-256 placeholders (update on release).
- **VS Code extension** (`plugins/vscode/`) — 6 commands, context menus, keybinding, esbuild pipeline → `out/extension.js`, `@vscode/vsce` packaging. **7 Jest tests passing.**
- **OneNote add-in** (`plugins/onenote/`) — HTTP-only bridge (browser sandbox), manifest.xml, task pane UI (`taskpane.html`), ribbon commands (`commands.html`), esbuild pipeline → `dist/`. **10 Jest tests passing.**

---

## Repository Structure

```
hitchmark/
├── crates/
│   ├── hitchmark-core/      # Core library (URI, purple, store) — 22 tests
│   ├── hitchmark-cli/       # `hk` binary (8 commands)
│   └── hitchmark-daemon/    # Linux DBus daemon
├── apps/
│   └── macos/               # SwiftUI menu bar app — 24 tests
├── plugins/
│   ├── obsidian/            # TypeScript Obsidian plugin — 12 tests
│   ├── vscode/              # VS Code extension — 7 tests
│   └── onenote/             # OneNote add-in — 10 tests
├── Formula/                 # Homebrew formula
├── specs/                   # Normative specifications (locked)
├── docs/src/                # mdBook source (12 pages)
├── .github/workflows/ci.yml # CI: rust, MSRV, deny, audit, node tests
├── deny.toml                # cargo-deny config
├── Cargo.toml               # Workspace root (rust-version = "1.75")
└── rust-toolchain.toml      # Pinned Rust version
```

---

## Test Coverage

| Suite | Count | Status |
|-------|-------|--------|
| Rust core (uri, purple, store) | 20 | ✅ passing |
| Rust CLI integration | 2 | ✅ passing |
| Obsidian plugin (Jest) | 12 | ✅ passing |
| macOS Swift | 24 | ✅ passing |
| VS Code extension (Jest) | 7 | ✅ passing |
| OneNote add-in (Jest) | 10 | ✅ passing |
| **Total** | **75** | **✅ all passing** |

---

## Key Architecture Decisions

| Decision | Rationale |
|----------|-----------|
| Rust core library | Type safety, zero unsafe, performance |
| SHA-256 + base58 IDs | Deterministic, human-readable, collision-resistant |
| SQLite bundled | Zero system dependencies, embeds everywhere |
| Subprocess bridge (not FFI) | Decoupled from CLI version; faster to ship |
| HTTP-first with subprocess fallback | Faster IPC when `hk serve` running; graceful degradation |
| XDG Base Directory | Cross-platform standard (`~/.config/hookmarks/`) |
| SwiftUI MenuBarExtra | Native, reactive, no AppKit boilerplate |
| CM6 ViewPlugin (Obsidian) | Live decorations without patching markdown source |
| OneNote HTTP-only | Browser sandbox cannot spawn subprocesses |
| `--json` on `hk list` | Structured output for all consumers |

---

## Dependencies

### Rust
`serde`, `serde_json`, `rusqlite` (bundled), `sha2`, `bs58`, `base64`, `chrono`, `regex`, `thiserror`, `anyhow`, `clap`, `clap_complete`, `opener`, `dirs`, `toml`, `log`, `env_logger`, `axum`, `tokio`, `tower-http`

### TypeScript (plugins)
`@noble/hashes` (SHA-256), `esbuild` (bundler), `jest` (tests), `@vscode/vsce` (VS Code packaging), `office-js` (OneNote, CDN)

### Swift
Built-in only: `SwiftUI`, `Cocoa`, `Foundation`, `ServiceManagement`

---

## Known Limitations

- Bookmark URI resolution not implemented (`UriType::Bookmark` returns "not yet implemented")
- `x-callback-url` not supported
- No code signing (can't distribute via Mac App Store or notarize)
- Linux daemon system tray (ksni) feature-flagged, not wired
- Native macOS OneNote app has no add-in API (Microsoft limitation) — add-in works on OneNote Online + Windows only
- Homebrew formula SHA-256 placeholders need updating after first GitHub release
- Windows path handling untested

---

## Roadmap

| Version | Features |
|---------|----------|
| **v0.1.0** ✅ | All 7 steps + hardening |
| **v0.2.0** 🚧 | `hk serve`, macOS prefs persistence, Swift tests, Homebrew, VS Code + OneNote extensions |
| v0.3.0 | Safari extension, code signing, Finder extension |
| v0.4.0 | Global hotkeys, web dashboard, Windows installer |
| v1.0.0 | Stable API, production release |

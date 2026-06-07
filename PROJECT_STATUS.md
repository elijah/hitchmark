# Project  HookmarksStatus 

**Last Updated**: After hardening pass (v0.1.0)
**Status All 7 blueprint steps complete + stability hardening merged**: 
**Build All tests passing, zero warnings**: 
**Branch**: `master`

---

## Completed Milestones

### Step  Monorepo Scaffold 0 
Cargo workspace (3 crates), GitHub Actions CI, governance docs (README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY), Rust 1.75+ toolchain, formatting/lint config.

### Step  Normative Specifications 1 
- `specs/uri-scheme. `hook://` URI grammar, validation, normalization, security model (v0.1, locked)md` 
- `specs/purple-numbers. SHA-256 + base58 paragraph IDs, collision handling, rendering rules (v0.1, locked)md` 

### Step  Core Library () 2 
| Module | Description |
|--------|-------------|
| `uri.rs` | `hook://` parser & validator |
 base58 ID generator, paragraph splitter |
| `store.rs` | SQLite link store (WAL, busy_timeout, schema versioning) |
| `error.rs` | Typed errors with `thiserror` |

**Tests**: 20 passing

### Step  CLI Tool () 3 
| Command | Description |
|---------|-------------|
| `hk link <a> <b> [--note]` | Create bidirectional link |
| `hk list <uri> [--json]` | Query links (plain or JSON) |
| `hk delete <a> <b> [-y]` | Remove a link |
| `hk open <uri>` | Resolve and open a hook:// URI |
 hook:// URI |
| `hk purple <file>` | Annotate file with stable paragraph IDs |

XDG-compliant config (`~/.config/hookmarks/`), panic-free error handling.

### Step  macOS SwiftUI App 4 
Menu bar app (`apps/ MenuBarExtra with 3-tab UI (Link / List / Finder), subprocess bridge to `hk`, preferences window (4 tabs), `hook://` URL scheme handler, AppleScript Finder/Safari bridges. Builds clean with Swift Package Manager (macOS 13+, zero external deps).macos/`) 

### Step  Linux Daemon 5 
`hookmarks-daemon`  zbus/DBus session service with 4 methods (`OpenUri`, `CreateLink`, `ListLinks`, `FileToUri`), systemd user service + security hardening, `install-linux.sh`, `#[cfg(target_os = "linux")]` gated (builds clean on macOS).crate 

### Step  Obsidian Plugin 6 
 `7nxxnx` in both runtimes. 12/12 Jest tests passing.

### Step  Documentation Site 7 
mdBook site (`docs/ 12 pages across 5 sections (Getting Started, URI Scheme, Purple Numbers, CLI Reference, Integration). GitHub Actions deploy-to-Pages workflow.src/`) 

### Hardening Pass 
- SQLite: WAL mode, 5s busy_timeout, `foreign_keys ON`, schema version table, migration guard
- Graceful `LinkAlreadyExists` error (no raw DB error surfaces to users)
- `hk delete` command with `--yes` flag
- `hk list --json` flag for machine-readable output
- Obsidian bridge updated to use `--json` (no more fragile tab-parsing)
- 9 new URI edge-case tests (unicode paths, spaces, bad base64, unknown authority, etc.)
- MSRV pinned: `rust-version = "1.75"` in workspace
- `deny. license allow-list, vuln deny, wildcard dep bantoml` 
- CI expanded: MSRV job, `cargo deny`, `cargo audit`, Node.js plugin test job

---

## Repository Structure

```
not-hookmarks/
 crates/
 hookmarks-core/          # Core library (URI, purple, store)   
 hookmarks-cli/           # `hk` binary (6 commands)   
 hookmarks-daemon/        # Linux DBus daemon   
 apps/
 macos/                   # SwiftUI menu bar app   
 plugins/
 obsidian/                # TypeScript Obsidian plugin   
 specs/                       # Normative specifications
 docs/
 src/                     # mdBook source (12 pages)   
 book/                    # Built site (gitignored)   
 .github/workflows/ci.yml     # CI: test, fmt, clippy, MSRV, deny, audit, node
 deny.toml                    # cargo-deny config
 Cargo.toml                   # Workspace root (rust-version = "1.75")
 rust-toolchain.toml          # Pinned Rust version
```

---

## Test Coverage

| Suite | Count | Status |
|-------|-------|--------|
|  (uri, purple, store) | 20 |  passing |
|  (integration) | 2 |  passing |
| Obsidian plugin (Jest) | 12 passing | | 
| macOS Swift | 0 | planned v0.2 |

---

## Key Architecture Decisions

| Decision | Rationale |
|----------|-----------|
| Rust core library | Type safety, zero unsafe, performance |
| SHA-256 + base58 IDs | Deterministic, human-readable, collision-resistant |
| SQLite bundled | Zero system dependencies, embeds everywhere |
| Subprocess bridge (not FFI) | Decoupled from CLI version; faster to ship |
| XDG Base Directory | Cross-platform standard (`~/.config/hookmarks/`) |
| SwiftUI MenuBarExtra | Native, reactive, no AppKit boilerplate |
| CM6 ViewPlugin (Obsidian) | Live decorations without patching markdown source |
| `--json` on `hk list` | Structured output for all consumers (bridges, scripts) |

---

## Dependencies

### Rust
`serde`, `serde_json`, `rusqlite` (bundled), `sha2`, `bs58`, `base64`, `chrono`, `regex`, `thiserror`, `anyhow`, `clap`, `opener`, `dirs`, `toml`, `log`, `env_logger`

### TypeScript (Obsidian plugin)
`@noble/hashes` (SHA-256), `esbuild` (bundler), `jest` (tests)

### Swift
Built-in only: `SwiftUI`, `Cocoa`, `Foundation`

---

## Known Limitations (v0.1)

- Bookmark URI resolution not implemented (`UriType::Bookmark` returns "not yet implemented")
- `x-callback-url` not supported (deferred to v0.2)
- macOS app preferences not persisted across restarts
- No global hotkey support yet
- No code signing (can't distribute to other Macs)
- Linux daemon system tray (ksni) feature-flagged, not wired
- No `hk serve` HTTP server (planned v0.2)
- Windows path handling untested

---

## Roadmap

| Version | Features |
|---------|----------|
| **v0.1.0** | All 7 steps + hardening | 
| v0.2.0 | `hk serve` HTTP server, macOS prefs persistence, global hotkeys |
| v0.3.0 | Code signing, Finder extension, Safari extension |
| v0.4.0 | VS Code extension, web dashboard |
| v1.0.0 | Stable API, production release |

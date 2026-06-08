# not-hookmarks

A cross-platform implementation of the Hookmarks protocol — stable, addressable links to files, web pages, emails, and intra-document locations using the `hook://` URI scheme.

## What is Hookmarks?

Hookmarks extends web browsers' `http://` URIs to local and structured documents:

- **Files**: `hook://file/<base64url-path>#para-abc123` → stable link to a paragraph in a local document
- **Bookmarks**: `hook://bookmark/<uuid>` → reference non-file resources (URLs, emails, note IDs)
- **Purple Numbers**: Margin-rendered paragraph IDs; clickable fragments that survive document reordering

Read the spec at [`specs/uri-scheme.md`](specs/uri-scheme.md).

## Components

| Component | Role | Status |
|-----------|------|--------|
| **hookmarks-core** | URI parser, storage engine, purple-number algorithm | ✅ v0.1.0 |
| **hookmarks-cli** (`hk`) | 7-command CLI tool, HTTP API server | ✅ v0.1.0 |
| **hookmarks-daemon** | Linux DBus service (systemd) | ✅ v0.1.0 |
| **hookmarks-macos** | SwiftUI menu-bar app | ✅ v0.1.0 |
| **hookmarks-obsidian** | Obsidian community plugin (CM6, 12 tests) | ✅ v0.1.0 |

## Installation

### Homebrew (coming soon)

```bash
brew tap yourusername/hookmarks
brew install hookmarks
```

### From source (requires Rust 1.75+)

```bash
cargo install --path crates/hookmarks-cli --locked
```

### Shell completions

```bash
# Bash
hk completions bash >> ~/.bashrc

# Zsh
hk completions zsh > "$(brew --prefix)/share/zsh/site-functions/_hk"

# Fish
hk completions fish > ~/.config/fish/completions/hk.fish
```

## Quick Start

```bash
# Convert a file to a hook:// URI
hk file ~/docs/note.md

# Link two documents together
hk link ~/docs/note.md ~/docs/reference.md --note "See section 3"

# List links for a file (JSON output)
hk list ~/docs/note.md --json

# Remove a link
hk delete ~/docs/note.md ~/docs/reference.md

# Start the HTTP API server (for Obsidian plugin, editor integrations)
hk serve
```

### Build from source

```bash
cargo build --release
```

### Test

```bash
cargo test --all          # Rust (22 tests)
cd plugins/obsidian && npm test  # TypeScript (12 tests)
cd apps/macos && swift test      # Swift (24 tests)
```

### Lint & Format

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

## Architecture

```
not-hookmarks/
├── specs/                       # Normative specs (URI scheme, purple numbers)
├── crates/
│   ├── hookmarks-core/          # Rust library (URI, storage, purple IDs)
│   ├── hookmarks-cli/           # CLI binary (hk)
│   └── hookmarks-daemon/        # Linux daemon
├── apps/
│   └── macos/                   # Swift package (menu bar app)
├── plugins/
│   └── obsidian/                # TypeScript plugin
├── Formula/                     # Homebrew formula
└── docs/                        # mdBook documentation site
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). We welcome bug reports, feature discussions, and pull requests.

## License

MIT. See [`LICENSE`](LICENSE).

## Design Principles

1. **No central server** — storage is local (SQLite) or cloud-sync (Obsidian, iCloud)
2. **Cross-platform** — works on macOS, Linux, and in browsers
3. **Stable identifiers** — IDs survive document reordering and minor edits
4. **Composable** — build on the spec; extend via plugins
5. **Privacy-first** — links are never transmitted unless explicitly shared

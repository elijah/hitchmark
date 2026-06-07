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
| **hookmarks-core** | URI parser, storage engine, purple-number algorithm | In progress |
| **hookmarks-cli** | `hk` command-line tool | Planned |
| **hookmarks-daemon** | Linux DBus service | Planned |
| **hookmarks-macos** | SwiftUI menu-bar app | Planned |
| **hookmarks-obsidian** | Obsidian community plugin | Planned |

## Quick Start

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test
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
│   ├── hookmarks-cli/           # CLI binary
│   └── hookmarks-daemon/        # Linux daemon
├── apps/
│   ├── macos/                   # Swift package
│   └── linux-tray/              # (Future)
├── plugins/
│   └── obsidian/                # TypeScript plugin
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

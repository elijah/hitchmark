# hitchmark-core

Core library for [Hitchmark](https://github.com/elijah/hitchmark) — stable, addressable document links via `hook://` URIs.

## Features

- **`hook://file/` URIs** — base64url-encoded absolute filesystem paths with optional `#para-<id>` fragments
- **`hook://bookmark/` URIs** — stable UUID-based references that survive file renames
- **Purple numbers** — SHA-256 content-hashed paragraph IDs (6–8 base58 chars) for intra-document linking
- **SQLite link store** — bidirectional link graph with GC, export, and import

## Usage

```toml
[dependencies]
hitchmark-core = "0.5"
```

```rust
use hitchmark_core::{LinkStore, HookUri};

let store = LinkStore::open("/path/to/store.db")?;
store.create_link("hook://file/...", "hook://bookmark/uuid", None)?;
let links = store.list_links_for("hook://file/...")?;
```

## License

MIT OR Apache-2.0

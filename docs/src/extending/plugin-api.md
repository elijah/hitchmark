# Plugin API

Hookmarks is designed to be extended. The cleanest integration point for
new clients (editors, apps, scripts) is the `hk` CLI.

## Subprocess integration (recommended)

The simplest approach: shell out to `hk`. All platform apps use this pattern.

```python
import subprocess, json

def file_to_uri(path: str) -> str:
    result = subprocess.run(
        ["hk", "file", path],
        capture_output=True, text=True, check=True
    )
    return result.stdout.strip()

def list_links(uri: str) -> list[dict]:
    result = subprocess.run(
        ["hk", "list", uri],
        capture_output=True, text=True, check=True
    )
    records = []
    for line in result.stdout.strip().splitlines():
        parts = line.split("\t")
        records.append({
            "source": parts[0] if len(parts) > 0 else "",
            "target": parts[1] if len(parts) > 1 else "",
            "note":   parts[2] if len(parts) > 2 else "",
        })
    return records
```

### Available subcommands

| Command | Output |
|---------|--------|
| `hk file <path>` | `hook://` URI (single line) |
| `hk link <a> <b> [--note text]` | confirmation message |
| `hk list <uri>` | tab-separated link records, one per line |
| `hk open <uri>` | opens file, no stdout |
| `hk purple <path> --format json` | JSON array of `{id, text, uri_fragment}` |

### Error handling

`hk` exits with code `0` on success, `1` on error. Errors are written to
stderr.

```bash
if ! hk file /nonexistent 2>/dev/null; then
    echo "File not found"
fi
```

## HTTP API (hk serve) — planned v0.2

In v0.2, `hk serve` will start a local HTTP server on `localhost:7878`.
This allows browser extensions and sandboxed environments (like Obsidian
mobile) to access the link store.

Planned endpoints:
```
POST /links          { source, target, note? }
GET  /links?uri=...  → [{ source, target, note, created_at }]
GET  /file?path=...  → { uri }
POST /open           { uri }
```

The Obsidian plugin already has the `daemonUrl` setting ready for this.

## DBus API (Linux)

On Linux, the daemon exposes a session bus service. See
[DBus Interface](./dbus.md) for the full reference.

## Rust library (`hookmarks-core`)

For Rust projects, you can depend on `hookmarks-core` directly:

```toml
[dependencies]
hookmarks-core = { git = "https://github.com/elw/not-hookmarks" }
```

Key types:
```rust
use hookmarks_core::{HookUri, UriType, LinkStore};
use hookmarks_core::purple::{PurpleNumberGenerator, split_paragraphs};

// Parse a URI
let uri = HookUri::parse("hook://file/L1Zvby9iYXI")?;

// Open the link store
let store = LinkStore::open("/path/to/store.db")?;

// Create a link
store.create_link("hook://file/...", "hook://file/...", Some("note"))?;

// Generate purple IDs
let mut gen = PurpleNumberGenerator::new();
let id = gen.generate("paragraph text")?;
```

## TypeScript / JavaScript

The purple number algorithm is available in the Obsidian plugin source as a
standalone module:

```typescript
import { generatePurpleId, splitParagraphs } from "./purple";

const id = generatePurpleId("paragraph text");  // → "7nxxnx"
const paras = splitParagraphs(markdownText);
```

The module has zero dependencies beyond `@noble/hashes` and can be extracted
for use in any JavaScript environment.

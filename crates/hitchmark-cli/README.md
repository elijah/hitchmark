# hitchmark-cli (`hk`)

CLI tool for [Hitchmark](https://github.com/elijah/hitchmark) — stable, addressable document links via `hook://` URIs.

## Install

```bash
# Homebrew (macOS / Linux)
brew install elijah/hitchmark/hitchmark

# cargo
cargo install hitchmark-cli
```

## Commands

| Command | Description |
|---------|-------------|
| `hk link <a> <b>` | Create a bidirectional link between two URIs or file paths |
| `hk list <uri>` | List all links for a resource |
| `hk delete <a> <b>` | Remove a link |
| `hk file <path>` | Print the `hook://` URI for a file |
| `hk open <uri>` | Resolve and open a `hook://` URI |
| `hk gc [--delete]` | Garbage-collect stale links |
| `hk export` | Export to NDJSON or JSON |
| `hk import <file>` | Import from NDJSON |
| `hk purple <file>` | Annotate with stable paragraph IDs |
| `hk serve` | Start local HTTP API (port 2701) |
| `hk watch` | Watch for file renames and auto-repair links |
| `hk version` | Print version info |
| `hk completions <shell>` | Print shell completion script |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `HK_STORE_PATH` | Override SQLite store path (useful for testing) |
| `HK_CONFIG_DIR` | Override config directory |

## License

MIT OR Apache-2.0

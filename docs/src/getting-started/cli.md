# CLI Reference

The `hk` command-line tool is the core of Hookmarks. All other platform apps
delegate to it.

## Installation

```bash
cargo install hookmarks-cli
```

Or build from source:
```bash
cargo build --release -p hookmarks-cli
```

## Commands

### `hk link` — Create a bidirectional link

```
hk link <uri-a> <uri-b> [--note <text>]
```

Links two resources together. Either argument can be a file path or a
`hook://` URI. File paths are automatically converted.

```bash
# Link two files
hk link ~/docs/project.md ~/docs/reference.md

# Link with a note
hk link ~/docs/project.md ~/research/paper.pdf \
    --note "Supporting evidence for section 4"

# Link using hook:// URIs directly
hk link "hook://file/L3Bhc..." "hook://file/L3Jhc..."

# Link to a specific paragraph
hk link ~/docs/note.md "hook://file/L3Jhc...#para-7nxxnx"
```

### `hk list` — Show links for a resource

```
hk list <uri>
```

Lists all bidirectional links for a file or URI. The output is
tab-separated: `source\ttarget\tnote\tcreated_at`.

```bash
# List links for a file
hk list ~/docs/project.md

# List links using hook:// URI
hk list "hook://file/L3Bhc..."
```

Example output:
```
hook://file/L3Bhc...    hook://file/L3Jhc...    Supporting evidence    2024-01-15T10:30:00Z
hook://file/L3Bhc...    hook://file/L2Zvb...                           2024-01-16T09:15:00Z
```

### `hk open` — Resolve and open a URI

```
hk open <hook-uri>
```

Resolves a `hook://` URI and opens the target using the system default
application (`xdg-open` on Linux, `open` on macOS).

```bash
# Open a file
hk open "hook://file/L3Vzc..."

# Open to a specific paragraph (opens file, browser scrolls to anchor)
hk open "hook://file/L3Vzc...#para-7nxxnx"
```

### `hk file` — Get the hook:// URI for a file

```
hk file <path>
```

Converts a file path to a `hook://` URI. Expands `~`, resolves relative
paths, and normalizes the result.

```bash
hk file ~/docs/project.md
# → hook://file/L1VzZXJzL2VsdykzZG9jcy9wcm9qZWN0Lm1k

hk file ./relative-path.md
# → hook://file/<absolute-base64-path>
```

### `hk purple` — Annotate a file with purple numbers

```
hk purple <path> [--format markdown|json]
```

Generates stable paragraph IDs for all paragraphs in a file.

```bash
# Markdown output (default)
hk purple ~/docs/note.md

# JSON output (for programmatic use)
hk purple ~/docs/note.md --format json
```

Markdown output:
```markdown
This is the first paragraph.
[§7nxxnx]

This is the second paragraph.
[§3qrstu]
```

JSON output:
```json
[
  {
    "id": "7nxxnx",
    "text": "This is the first paragraph.",
    "uri_fragment": "para-7nxxnx"
  }
]
```

## Configuration

Hookmarks stores its database and config in `~/.config/hookmarks/`:

```
~/.config/hookmarks/
├── config.toml    # Configuration file
└── store.db       # SQLite link database
```

### config.toml

```toml
# Path to the SQLite database
store_path = "/home/you/.config/hookmarks/store.db"

# Automatically open links when resolved
auto_open = true

# Default note template for new links (supports {date}, {source}, {target})
note_template = ""
```

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `warn` | Log level: `error`, `warn`, `info`, `debug`, `trace` |
| `HOOKMARKS_STORE` | `~/.config/hookmarks/store.db` | Override database path |

## Shell completions

Generate completion scripts for your shell:

```bash
# Bash
hk --generate-completion bash >> ~/.bashrc

# Zsh
hk --generate-completion zsh >> ~/.zshrc

# Fish
hk --generate-completion fish > ~/.config/fish/completions/hk.fish
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (see stderr) |
| 2 | Invalid arguments |
| 3 | Resource not found |

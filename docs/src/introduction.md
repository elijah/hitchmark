# Hookmarks

> **Stable, addressable links to documents and paragraphs.**

Hookmarks lets you create permanent, bidirectional links between *any* documents —
Markdown files, PDFs, web pages, emails, source code, and more. Links survive
file moves, renames, and paragraph reordering.

## What makes Hookmarks different?

| Feature | Hookmarks | Regular bookmarks | Wiki links |
|---------|-----------|-------------------|------------|
| Bidirectional | ✅ | ❌ | Some |
| Survives file moves | ✅ | ❌ | ❌ |
| Paragraph-level precision | ✅ | ❌ | ❌ |
| Cross-app (Finder, Obsidian, email) | ✅ | ❌ | ❌ |
| Open standard URI scheme | ✅ | ❌ | ❌ |
| Works offline | ✅ | ✅ | ✅ |

## Core concepts

**`hook://` URIs** — a custom URI scheme that encodes a stable address to any
resource. File paths are base64-encoded so they survive spaces and special
characters. Paragraph addresses are appended as `#para-<id>` fragments.

```
hook://file/L1VzZXJzL2Vsd...#para-7nxxnx
       │    │                  └─ Purple number (paragraph ID)
       │    └─ Base64url-encoded absolute path
       └─ URI type (file | bookmark | x-callback-url)
```

**Purple numbers** — stable, hash-based IDs for individual paragraphs.
Generated from the paragraph's content using SHA-256 → base58. They survive
paragraph reordering and minor edits.

```markdown
This is a paragraph with a stable purple number. §7nxxnx
```

**Bidirectional links** — every link has two ends. When you create a link
between document A and document B, both documents "know" about the connection.
Query either end to discover the other.

## Architecture overview

```
┌──────────────────────────────────────────────────┐
│                  Applications                    │
│  macOS app   │  Obsidian plugin  │  CLI (hk)     │
└──────┬───────┴──────────┬────────┴───────┬───────┘
       │                  │                │
       └──────────────────┼────────────────┘
                          │ subprocess / DBus
                   ┌──────┴───────┐
                   │ hitchmark-core│
                   │  (Rust lib)  │
                   └──────┬───────┘
                          │
                   ┌──────┴───────┐
                   │   SQLite DB  │
                   │ store.db     │
                   └──────────────┘
```

## Quick start

Pick your platform:

- **[macOS →](./getting-started/macos.md)** — menu bar app + CLI
- **[Linux →](./getting-started/linux.md)** — daemon + CLI
- **[CLI only →](./getting-started/cli.md)** — cross-platform terminal tool
- **[Obsidian →](./getting-started/obsidian.md)** — community plugin

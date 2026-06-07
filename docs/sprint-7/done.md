# Step 7: Documentation Site — Done ✅

**Branch:** `feature/step-5-and-7`  
**Commit:** `b6b8ac7`

## Delivered

### 15 documentation files + CI workflow

| Section | Pages | Content |
|---------|-------|---------|
| Introduction | 1 | Overview, architecture diagram, feature comparison table |
| Getting Started | 5 | Installation, macOS, Linux, CLI reference, Obsidian |
| Concepts | 2 | URI Scheme reference, Purple Numbers deep-dive |
| Extending | 2 | Plugin API, DBus interface reference |
| Project | 2 | Contributing guide, Changelog |

### Build
- `mdbook build docs/` → `docs/book/` ✅ zero errors
- GitHub Pages deploy workflow: `.github/workflows/deploy-docs.yml`
- Triggers on push to master when `docs/**` changes

### Highlights
- **CLI reference** (cli.md): complete `hk` command docs with examples, exit codes, shell completions
- **URI scheme** (uri-scheme.md): full ABNF grammar, normalization rules, decode examples in Python/JS/Rust
- **Purple numbers** (purple-numbers.md): algorithm, stability table, ARIA accessibility
- **Plugin API** (plugin-api.md): subprocess, HTTP (planned v0.2), Rust crate, TypeScript module
- **DBus** (dbus.md): method signatures, gdbus examples, Python dbus example

## Decisions made
- Used mdBook 0.4.40 (latest stable)
- No custom theme (uses built-in light/navy) — keeps maintenance surface small
- `edit-url-template` wired to GitHub edit links for community contributions
- Build output (`docs/book/`) gitignored — CI builds fresh on each push

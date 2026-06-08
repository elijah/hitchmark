# Contributing

Thank you for considering a contribution to Hookmarks!

## Ways to contribute

- **Bug reports** — open an issue with reproduction steps
- **Feature requests** — open an issue with the use case
- **Code** — fork, branch, implement, and open a PR
- **Documentation** — edit files in `docs/src/` and open a PR
- **Translations** — help translate the docs (future)

## Development setup

```bash
# Clone
git clone https://github.com/elw/hitchmark
cd hitchmark

# Install Rust (https://rustup.rs)
rustup toolchain install stable

# Install Node.js (https://nodejs.org) for the Obsidian plugin

# Build everything
cargo build --all
cd plugins/obsidian && npm install && npm run build
cd apps/macos && swift build
```

## Project structure

```
hitchmark/
├── crates/
│   ├── hitchmark-core/     # Core library (all shared logic)
│   ├── hitchmark-cli/      # hk binary
│   └── hitchmark-daemon/   # Linux DBus daemon
├── apps/
│   ├── macos/              # SwiftUI menu bar app
│   └── linux-tray/         # .desktop, systemd unit
├── plugins/
│   └── obsidian/           # TypeScript Obsidian plugin
├── specs/                  # Normative specifications
├── docs/                   # This documentation site
└── scripts/                # install-linux.sh
```

## Code standards

### Rust
- `cargo fmt --all` — code must be formatted
- `cargo clippy --all -- -D warnings` — zero clippy warnings
- `cargo test --all` — all tests must pass
- `#![forbid(unsafe_code)]` — no unsafe Rust (workspace-wide)
- Document all public items (`missing_docs = "warn"`)

### Swift
- `swift build` — must compile without errors
- Follow existing code patterns

### TypeScript
- `npm run build` — zero TypeScript errors
- `npm test` — all Jest tests must pass
- Prefer strict TypeScript (noImplicitAny, strictNullChecks)

## Commit style

```
type: short description (Fixes #42)

Longer explanation if needed.

Co-authored-by: Your Name <you@example.com>
```

Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `style`

Use [GitHub closing keywords](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue)
in commits: `Fixes #42`, `Closes #43`.

## Pull request process

1. Fork the repository
2. Create a branch: `git checkout -b feature/my-feature`
3. Make changes and commit
4. Push: `git push origin feature/my-feature`
5. Open a PR against `master`

PRs should:
- Have a clear title and description
- Reference any related issues
- Pass CI checks
- Include tests for new functionality
- Update documentation if behavior changes

## Specifications

The `specs/` directory contains normative specifications for the `hook://`
URI scheme and purple numbers. **Do not implement changes that conflict with
the specs without updating the spec first** and getting agreement.

## Architecture decision records

Significant decisions are documented in commit messages and `docs/sprint-*/done.md`.
When making a significant architectural change, document the rationale.

## Code of conduct

See [CODE_OF_CONDUCT.md](../../../CODE_OF_CONDUCT.md). We follow the
Contributor Covenant.

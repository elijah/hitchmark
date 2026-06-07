# Contributing to not-hookmarks

Thank you for your interest! We're building a cross-platform, open-source implementation of Hookmarks. All contributions — code, documentation, design, and discussion — are welcome.

## Getting Started

1. **Fork and clone** the repo
2. **Create a branch**: `git checkout -b feature/your-feature` or `fix/your-bug`
3. **Build and test**: `cargo build && cargo test`
4. **Commit with clear messages**: use conventional commits (e.g., `feat: add purple-number CLI`)
5. **Push and open a PR**: describe your changes and link any relevant issues

## Development Workflow

### Prerequisites

- Rust 1.75+ (see `rust-toolchain.toml`)
- `rustfmt` and `clippy` (installed with Rust)

### Commands

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Test
cargo test

# Build docs (if you edit specs/)
cargo doc --no-deps --open
```

### Code Style

- No `unsafe` code without explicit review (workspace forbids it by default)
- Prefer clear, boring code over clever code
- Add tests for all public APIs
- Comment only what isn't obvious from the code itself

## PR Guidelines

- **One concern per PR** — split refactors from feature work
- **Test your changes** — CI runs `cargo test`, `cargo clippy`, and `cargo fmt --check`
- **Update docs** — if you change public APIs or add features, document them
- **Link issues** — use GitHub's `Closes #N` or `Fixes #N` keywords

## Specs & Design

Changes to the URI scheme or purple-number algorithm are high-impact. If you're considering a change:

1. **Open an issue first** — discuss the problem and proposed solution
2. **Update the spec** — changes to `specs/*.md` require broader review
3. **Add tests** — ensure your implementation matches the spec

## Testing Strategy

### Unit Tests

All crate-level functionality has unit tests:
- `hookmarks-core`: URI parsing, purple-ID generation, storage operations
- `hookmarks-cli`: argument parsing, command behavior
- `hookmarks-daemon`: DBus interface (Linux only)

### Integration Tests

- End-to-end `hk` CLI workflows (file → URI → storage → resolve)
- Cross-platform: tests run on macOS and Linux in CI

### Manual Testing

Before opening a PR, test manually:
- Build and run `hk` — create a test link, resolve it
- On macOS: confirm the app launches and registers the `hook://` scheme
- On Linux: confirm `xdg-open hook://...` works

## Reporting Issues

- **Bug**: describe steps to reproduce, expected behavior, actual behavior
- **Feature request**: explain the use case and why this feature matters
- **Documentation**: link the page and explain what's unclear

## Questions?

Open an issue or discussion. The maintainers are happy to help.

---

**Code of Conduct**: See [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Be respectful and inclusive.

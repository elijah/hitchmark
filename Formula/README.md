# Homebrew Tap — Hitchmark

This directory contains the Homebrew formula for the `hk` CLI tool.

## Installing via Homebrew

Once this tap is published:

```bash
brew tap elijah/hitchmark
brew install hitchmark
```

Or install directly from the formula:

```bash
brew install elijah/hitchmark/hitchmark
```

## Building bottles

After tagging a release on GitHub:

1. Update the `url` and `sha256` in `Formula/hitchmark.rb`:
   ```bash
   curl -L https://github.com/elijah/hitchmark/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256
   ```

2. Build the bottle on each platform:
   ```bash
   brew install --build-bottle hitchmark
   brew bottle hitchmark
   ```

3. Upload bottle tarballs to the GitHub release and update the `bottle do` block.

## Formula notes

- Builds only `hitchmark-cli` (`hk` binary) — not the macOS app or Linux daemon
- Uses `--locked` to ensure reproducible builds from `Cargo.lock`
- Installs shell completions for bash, zsh, and fish automatically
- Requires Rust (installed automatically by Homebrew as a build dependency)

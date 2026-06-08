# Homebrew Tap — Hookmarks

This directory contains the Homebrew formula for the `hk` CLI tool.

## Installing via Homebrew

Once this tap is published:

```bash
brew tap yourusername/hookmarks
brew install hookmarks
```

Or install directly from the formula:

```bash
brew install yourusername/hookmarks/hookmarks
```

## Building bottles

After tagging a release on GitHub:

1. Update the `url` sha256 in `Formula/hookmarks.rb`:
   ```bash
   curl -L https://github.com/yourusername/hitchmark/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256
   ```

2. Build the bottle on each platform:
   ```bash
   brew install --build-bottle hookmarks
   brew bottle hookmarks
   ```

3. Upload bottle tarballs to the GitHub release and update the `bottle do` block.

## Formula notes

- Builds only `hitchmark-cli` (`hk` binary) — not the macOS app or Linux daemon
- Uses `--locked` to ensure reproducible builds from `Cargo.lock`
- Installs shell completions for bash, zsh, and fish automatically
- Requires Rust (installed automatically by Homebrew as a build dependency)

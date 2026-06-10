# Installation

Hitchmark is available on macOS, Linux, and Windows. Choose the guide for your platform:

| Platform | Guide |
|----------|-------|
| macOS 13+ | [macOS Installation](./macos.md) |
| Linux (Ubuntu, Fedora, Arch) | [Linux Installation](./linux.md) |
| Windows 10/11 | [Windows Installation](./windows.md) |
| CLI only (any platform) | [CLI Reference](./cli.md) |
| Obsidian | [Obsidian Plugin](./obsidian.md) |
| Neovim | [Neovim Plugin](./neovim.md) |

## Prerequisites

All platforms require the `hk` CLI. The platform-specific apps depend on it.

### Install hk via Homebrew (macOS/Linux)

```bash
brew install elijah/hitchmark/hitchmark
```

### Install hk via Cargo (all platforms)

```bash
cargo install hitchmark-cli
```

### From source

```bash
git clone https://github.com/elijah/hitchmark
cd hitchmark
cargo build --release -p hitchmark-cli
cp target/release/hk ~/.local/bin/hk   # Linux
cp target/release/hk /usr/local/bin/hk # macOS
# Windows: copy target\release\hk.exe to a directory on %PATH%
```

### Verify installation

```bash
hk --version
# hk 0.3.0
```

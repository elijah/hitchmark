# Installation

Hookmarks is available on macOS and Linux. Choose the guide for your platform:

| Platform | Guide |
|----------|-------|
| macOS 13+ | [macOS Installation](./macos.md) |
| Linux (Ubuntu, Fedora, Arch) | [Linux Installation](./linux.md) |
| CLI only (any platform) | [CLI Reference](./cli.md) |
| Obsidian | [Obsidian Plugin](./obsidian.md) |

## Prerequisites

All platforms require the `hk` CLI. The platform-specific apps depend on it.

### Install hk via Cargo (all platforms)

```bash
cargo install hitchmark-cli
```

### From source

```bash
git clone https://github.com/elw/hitchmark
cd hitchmark
cargo build --release -p hitchmark-cli
cp target/release/hk ~/.local/bin/hk   # Linux
cp target/release/hk /usr/local/bin/hk # macOS
```

### Verify installation

```bash
hk --version
# hk 0.1.0
```

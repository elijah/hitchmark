# Linux

## Requirements

- Linux (Ubuntu 22.04+, Fedora 38+, Arch Linux)
- Rust toolchain
- `xdg-utils` package
- systemd (optional, for auto-start)

## Install

```bash
# 1. Build everything
git clone https://github.com/elw/hitchmark
cd hitchmark
cargo build --release -p hitchmark-cli -p hitchmark-daemon

# 2. Run the install script
./scripts/install-linux.sh
```

The install script:
- Copies `hitchmark-daemon` to `~/.local/bin/`
- Registers the `hook://` URI scheme via `xdg-mime`
- Installs and starts the systemd user service

## Manual installation

```bash
# Install binary
install -m 755 target/release/hitchmark-daemon ~/.local/bin/

# Register hook:// URI scheme
install -m 644 apps/linux-tray/hitchmark.desktop \
    ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/
xdg-mime default hitchmark.desktop x-scheme-handler/hook

# Install systemd service
mkdir -p ~/.config/systemd/user
sed "s|%h|$HOME|g" apps/linux-tray/hitchmark-daemon.service \
    > ~/.config/systemd/user/hitchmark-daemon.service
systemctl --user daemon-reload
systemctl --user enable --now hitchmark-daemon
```

## Verify

```bash
# Check daemon status
systemctl --user status hitchmark-daemon

# Verify URI scheme
xdg-mime query default x-scheme-handler/hook
# → hitchmark.desktop

# Test DBus
gdbus call --session \
  --dest org.hitchmark.Daemon \
  --object-path /org/hitchmark/Daemon \
  --method org.hitchmark.Daemon1.FileToUri \
  '/home/you/document.md'
```

## Using xdg-open

Once installed, `xdg-open` (and anything that calls it) can open `hook://` URIs:

```bash
xdg-open "hook://file/$(echo -n /home/you/document.md | base64 -w0)"
```

## DBus interface

The daemon exposes a DBus session service at `org.hitchmark.Daemon`.

| Method | Signature | Description |
|--------|-----------|-------------|
| `OpenUri` | `(s) → s` | Resolve and open a hook:// URI |
| `CreateLink` | `(sss) → s` | Create a bidirectional link |
| `ListLinks` | `(s) → as` | List all links for a URI |
| `FileToUri` | `(s) → s` | Convert file path to hook:// URI |

See [DBus Interface](../extending/dbus.md) for full documentation.

## Uninstall

```bash
./scripts/install-linux.sh --uninstall
```

## Troubleshooting

**Daemon won't start**
```bash
journalctl --user -u hitchmark-daemon -n 50
```

**hook:// links not opening**
```bash
xdg-mime query default x-scheme-handler/hook
# Should print: hitchmark.desktop
# If not, re-run: xdg-mime default hitchmark.desktop x-scheme-handler/hook
```

**DBus call fails**
- Ensure daemon is running: `systemctl --user is-active hitchmark-daemon`
- Check logs: `journalctl --user -u hitchmark-daemon`

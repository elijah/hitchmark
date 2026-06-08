# Hookmarks — Linux XDG Desktop Actions

Right-click context menu actions for Linux file managers, equivalent to the macOS System Services.

**Supported file managers:** Nautilus (GNOME), Dolphin (KDE), Thunar (Xfce), Nemo (Cinnamon/Mint)

## Actions

| Action | Description |
|--------|-------------|
| **Copy hook:// URI** | Convert selected file to a `hook://` URI → copy to clipboard |
| **Link Two Files** | Select 2 files → create a bidirectional Hookmarks link |
| **Show Links** | Display all documents linked to a file in a dialog |
| **Open hook:// URI** | Read `hook://` URI from clipboard → open the linked document |

## Quick Install

```sh
cd apps/linux
./install.sh
```

The installer auto-detects your desktop environment and installs only what's relevant. Restart your file manager after installation.

```sh
./install.sh --dry-run     # preview what would be installed
./install.sh --uninstall   # remove all installed files
```

## How It Works

```
apps/linux/
├── scripts/                    # Core shell scripts (DE-agnostic logic)
│   ├── hk-common.sh            # Shared: clipboard, notify, hk server probe
│   ├── hk-copy-uri.sh          # Convert file → URI → clipboard
│   ├── hk-link-files.sh        # Link 2 files
│   ├── hk-show-links.sh        # Show links dialog
│   └── hk-open-uri.sh          # Open hook:// URI from clipboard
├── nautilus/                   # GNOME Nautilus Scripts
├── kde/hookmarks.desktop       # KDE Dolphin ServiceMenu
├── thunar/uca-hookmarks.xml    # Xfce Thunar Custom Actions (merged)
├── nemo/                       # Cinnamon Nemo Actions
└── install.sh                  # Unified installer
```

The installer:
1. Copies core scripts to `~/.local/share/hookmarks/scripts/`
2. Creates `hookmarks-*` symlinks in `~/.local/bin/` (used by all DEs)
3. Installs DE-specific files for detected desktop environments

## Requirements

- `hk` CLI in `$PATH` (or `~/.local/bin/hk`, `~/.cargo/bin/hk`)
- Optional: `hk serve` running for faster HTTP transport
- For clipboard: `wl-clipboard` (Wayland) or `xclip`/`xsel` (X11)
- For dialogs: `zenity` (GNOME/GTK) or `kdialog` (KDE) — falls back to `notify-send`

```sh
# Wayland clipboard
sudo apt install wl-clipboard       # Debian/Ubuntu
sudo dnf install wl-clipboard       # Fedora

# X11 clipboard
sudo apt install xclip              # Debian/Ubuntu

# Dialogs (usually pre-installed with your DE)
sudo apt install zenity             # GNOME/GTK
sudo apt install kdialog            # KDE
```

## Manual Installation by DE

### Nautilus (GNOME)

```sh
mkdir -p ~/.local/share/nautilus/scripts
cp nautilus/* ~/.local/share/nautilus/scripts/
chmod +x ~/.local/share/nautilus/scripts/Hookmarks*
```

### KDE Dolphin (Plasma 5)

```sh
mkdir -p ~/.local/share/kservices5/ServiceMenus
cp kde/hookmarks.desktop ~/.local/share/kservices5/ServiceMenus/
kbuildsycoca5 --noincremental
```

### KDE Dolphin (Plasma 6)

```sh
mkdir -p ~/.local/share/kio/servicemenus
cp kde/hookmarks.desktop ~/.local/share/kio/servicemenus/
kbuildsycoca6 --noincremental
```

### Thunar (Xfce)

Edit `~/.config/Thunar/uca.xml` and paste the `<action>` blocks from `thunar/uca-hookmarks.xml` inside the `<actions>` element. (The installer does this merge automatically.)

### Nemo (Cinnamon/Mint)

```sh
mkdir -p ~/.local/share/nemo/actions
cp nemo/*.nemo_action ~/.local/share/nemo/actions/
```

## Open hook:// URI — Keyboard Shortcut

Add a keyboard shortcut in your DE settings to run `hookmarks-open-uri`. This lets you open any `hook://` URI you've copied to the clipboard without touching the file manager:

- **GNOME**: Settings → Keyboard → Custom Shortcuts → `hookmarks-open-uri`
- **KDE**: System Settings → Shortcuts → Custom Shortcuts → `hookmarks-open-uri`
- **Xfce**: Settings Manager → Keyboard → Application Shortcuts → `hookmarks-open-uri`

# Obsidian Plugin

The Hookmarks Obsidian plugin adds purple number annotations, bidirectional
link management, and `hook://` URI support to your Obsidian vault.

## Requirements

- Obsidian 1.4.0 or later (desktop)
- `hk` CLI installed (the plugin delegates to it)
- macOS or Linux

## Installation

### Via BRAT (beta testing)

1. Install [BRAT](https://github.com/TfTHacker/obsidian42-brat) from the
   Community Plugins browser
2. Open BRAT settings → Add Beta Plugin
3. Enter: `https://github.com/elw/hitchmark`
4. Click **Add Plugin**
5. Enable **Hookmarks** in Community Plugins

### Manual installation

```bash
# Build the plugin
cd plugins/obsidian
npm install
npm run build

# Copy to your vault
cp -r . "/path/to/your/vault/.obsidian/plugins/hookmarks/"
```

Then enable **Hookmarks** in Obsidian Settings → Community Plugins.

## Features

### Purple Numbers

Purple numbers appear as small `§abc123` annotations beside each paragraph
in the live preview editor. Click any annotation to copy the paragraph's
`hook://` URI to your clipboard.

```markdown
This is a paragraph. §7nxxnx
```

The `§id` is rendered in the editor margin — it doesn't appear in exported
or published documents.

**Settings:**
- Toggle visibility (Settings → Hookmarks → Show purple numbers)
- Change color (Settings → Hookmarks → Annotation color)
- Copy on click (Settings → Hookmarks → Copy URI on click)

### Commands

Access all commands via the Command Palette (`Cmd/Ctrl+P`):

| Command | Description |
|---------|-------------|
| Copy hook:// URI for active note | Copies `hook://file/<path>` |
| Copy hook:// URI for current paragraph | Copies `hook://file/<path>#para-<id>` |
| Create link: active note ↔ clipboard URI | Creates a bidirectional link |
| Open linked documents panel | Opens the link sidebar |
| Refresh link panel | Reloads links for current file |

### Link Panel

The link panel (sidebar) shows all bidirectional links for the active note.

- Click a link to open the connected document
- Right-click for context menu (Copy URI, Open)
- Click ↺ to refresh

Open it: Ribbon icon (🔗) or Command Palette → "Open linked documents panel"

## Configuration

Open Settings → Hookmarks:

### Purple Numbers
- **Show purple numbers** — toggle §id annotations in the editor
- **Annotation color** — CSS color for annotations (default: `#888`)
- **Copy URI on click** — click §id to copy paragraph URI

### CLI Integration
- **Path to hk binary** — leave blank to auto-detect in standard locations
- **Test connection** — verify `hk` is reachable

### Advanced
- **Daemon URL** — URL for `hk serve` HTTP server (optional)

## How the bridge works

The plugin runs the `hk` CLI via Node.js `child_process`. It auto-locates
the binary in:

1. `/usr/local/bin/hk`
2. `~/.cargo/bin/hk`
3. `/opt/homebrew/bin/hk` (macOS ARM64)

All operations are asynchronous and run off the main thread.

## Troubleshooting

**Purple numbers don't appear**
- Ensure "Show purple numbers" is enabled in Settings → Hookmarks
- Restart Obsidian (required after toggling the setting)

**"hk binary not found"**
- Run `which hk` in Terminal to find the path
- Set it in Settings → Hookmarks → CLI Integration → Path to hk binary

**Link panel shows no links**
- Make sure you've created at least one link with `hk link` or the Link tab
- Click ↺ to refresh

**"Create link" command does nothing**
- Copy a `hook://` URI to clipboard first, then run the command
- The command links the active note to whatever `hook://` URI is on your clipboard

# macOS

## Requirements

- macOS 13 (Ventura) or later
- Xcode Command Line Tools: `xcode-select --install`
- Rust toolchain (for building from source)

## Install the CLI

```bash
# From source
git clone https://github.com/elw/hitchmark
cd hitchmark
cargo build --release -p hitchmark-cli
cp target/release/hk /usr/local/bin/hk
```

Verify:
```bash
hk --version
```

## Build and run the menu bar app

```bash
cd apps/macos
swift build
./.build/debug/Hookmarks
```

Or open in Xcode:
```bash
swift package generate-xcodeproj
open Hookmarks.xcodeproj
```

The **Hookmarks** menu bar icon (🔗) will appear in your menu bar.

## Register the hook:// URL scheme

The macOS app registers itself as the `hook://` handler via `Info.plist`
when installed. If you're running from the build output, you can test
the URL scheme handler from Terminal:

```bash
open "hook://file/L1VzZXJzL2Vsd"
```

The app's `AppDelegate` intercepts the URL and calls `hk open <uri>`.

## Usage

### Create a link between two files

1. Open the menu bar icon
2. Go to the **Link** tab
3. Enter two file paths or `hook://` URIs
4. Click **Create Link**

Or from the CLI:
```bash
hk link ~/docs/note.md ~/docs/reference.md --note "See section 3"
```

### Get the hook:// URI for a Finder selection

1. Select a file in Finder
2. Click the menu bar icon
3. Go to the **Finder** tab
4. Click **Get Selected File**
5. Copy the generated URI

### Open a hook:// URI

Double-click any `hook://` link in a document — the app intercepts it and
opens the target file.

Or from the CLI:
```bash
hk open "hook://file/L1VzZXJzL2Vsd#para-7nxxnx"
```

## Troubleshooting

**"hk binary not found"**
- Ensure `hk` is installed: `which hk`
- Set the path in Preferences → CLI

**AppleScript permission denied (Finder tab)**
- Go to System Preferences → Privacy & Security → Automation
- Enable Hookmarks → Finder

**hook:// links don't open the app**
- Run the app at least once from Xcode or `./.build/debug/Hookmarks`
- The URL scheme is registered on first launch

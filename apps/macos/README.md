# Hookmarks macOS App

**Status:** Step 4 MVP Scaffold (Ready for Xcode build)

## Overview

This is a native macOS SwiftUI application that provides a menu bar interface to Hookmarks, integrating with the core Rust CLI (`hk`) to create and manage stable, addressable links to documents and paragraphs.

## Architecture

```
apps/macos/
├── Package.swift           # Swift Package manifest
├── Info.plist             # App bundle configuration
├── Hookmarks/
│   ├── App/
│   │   ├── HookmarksApp.swift      # @main entry point
│   │   └── AppDelegate.swift       # Lifecycle, URL scheme handling
│   ├── MenuBar/
│   │   └── MenuBarView.swift       # Menu bar UI (3 tabs: Link, List, Finder)
│   ├── Bridges/
│   │   ├── HKBridge.swift          # Subprocess bridge to `hk` CLI
│   │   └── FinderBridge.swift      # AppleScript bridge to Finder
│   ├── Preferences/
│   │   └── PreferencesView.swift   # Settings window (4 tabs)
│   └── Models/                     # (empty, ready for data models)
└── HookmarksTests/                 # (empty, ready for tests)
```

## Features Implemented

### MVP (Phase 1)
- [x] Menu bar icon with menu
- [x] **Link Tab**: Create bidirectional links between two URIs/paths
- [x] **List Tab**: Query existing links for a URI
- [x] **Finder Tab**: Get selected file from Finder and generate hook:// URI
- [x] Subprocess integration: Calls `hk` CLI for all operations
- [x] AppleScript bridge: Interacts with Finder
- [x] Preferences window with 4 tabs:
  - General (auto-open, launch at login)
  - Hotkeys (global hotkey configuration)
  - CLI (path configuration)
  - About (links and version)
- [x] URL scheme handler: `hook://` URIs from Safari/email open in the app

### Phase 2 (Optional, Deferred)
- [ ] Global hotkey support (Ctrl+Option+H)
- [ ] Finder extension (integration with Finder context menu)
- [ ] Safari extension (quick link creation)
- [ ] Mail bridge (link support in email)

### Phase 3 (Nice-to-have)
- [ ] Sparkle auto-update
- [ ] Code signing and notarization
- [ ] Dark mode refinements
- [ ] Undo/redo for links

## Building with Xcode

### Prerequisites
1. **Xcode 15+** (with command line tools)
2. **Rust toolchain** (for building the `hk` CLI)
3. **hk CLI installed** in one of these locations:
   - `/usr/local/bin/hk`
   - `~/.cargo/bin/hk`
   - `/opt/homebrew/bin/hk`

### Step 1: Build the Rust CLI (if not already done)

```bash
cd /Users/elw/hitchmark
cargo build --release -p hitchmark-cli
```

The binary will be at `target/release/hk`. Install it:

```bash
cp target/release/hk /usr/local/bin/hk
chmod +x /usr/local/bin/hk
```

Or with Homebrew (when available):
```bash
brew install hookmarks
```

### Step 2: Generate Xcode Project

From the `apps/macos` directory:

```bash
cd apps/macos
swift package generate-xcodeproj
```

This creates `Hookmarks.xcodeproj`. If Xcode complains about compatibility, update the package:

```bash
swift package update
```

### Step 3: Open in Xcode

```bash
open Hookmarks.xcodeproj
```

### Step 4: Configure Build Settings

**Important:** The Swift Package Manager target needs the correct product type. In Xcode:

1. Select `Hookmarks` target
2. **Build Phases** tab → Verify `Link Binary With Libraries` is empty (SwiftUI is built-in)
3. **Info** tab → Set `Product Name` to `Hookmarks`
4. **Build Settings** tab → Search `MACOSX_DEPLOYMENT_TARGET` → Set to `13.0` or higher

### Step 5: Build & Run

```bash
# In Xcode: Cmd+B to build, Cmd+R to run
# Or from command line:
swift build
swift run Hookmarks
```

## Testing

Unit and integration tests will be added in Phase 1.1:

```bash
swift test
```

## Subprocess Integration

The app calls the `hk` CLI via subprocess (Process API):

```swift
let process = Process()
process.executableURL = URL(fileURLWithPath: "/usr/local/bin/hk")
process.arguments = ["open", "hook://example.com#para-42"]
```

**Key design choices:**
- Subprocess over FFI (simpler, faster to ship; FFI is v0.2)
- Auto-locates `hk` in standard paths
- Async subprocess calls to prevent UI blocking
- JSON output from `hk` for structured data (v0.2)

## AppleScript Bridge

The app uses AppleScript to interact with Finder:

```swift
let script = """
tell application "Finder"
    return POSIX path of (item 1 of selection as alias)
end tell
"""
NSAppleScript(source: script).executeAndReturnError()
```

**Permissions required:**
- `NSAppleEventsUsageDescription` in Info.plist (already set)
- User grant automation access on first use

## Menu Bar UX

The menu bar icon is a `.link.circle.fill` SF Symbol (blue). Clicking opens a menu with:

- 3 segmented tabs (Link, List, Finder)
- Input fields and buttons
- Real-time error display
- Settings button (⚙️)

## Preferences

- **General**: Auto-open links, launch at login
- **Hotkeys**: Global hotkey config (experimental)
- **CLI**: Path to `hk` binary
- **About**: Links to GitHub, docs, issue tracker

## Troubleshooting

### "hk command not found"
- Ensure `hk` is built: `cargo build --release -p hitchmark-cli`
- Install to `/usr/local/bin/hk`
- Update CLI path in Preferences → CLI

### AppleScript fails (Finder bridge)
- Go to **System Preferences** → **Privacy & Security** → **Automation**
- Grant Hookmarks access to Finder
- Restart the app

### Menu bar icon doesn't appear
- Check that the app is in `/Applications` or built with correct bundle ID
- Try relaunching: `killall Hookmarks && open Hookmarks.app`

### Subprocess hangs
- Check that `hk` is responsive: `hk list hook://example.com`
- Restart the macOS app
- File an issue at https://github.com/elw/hitchmark/issues

## Future Work

### Phase 2: AppleScript Bridges
- **Finder integration**: Right-click context menu
- **Safari integration**: Quick link button in toolbar
- **Mail integration**: Link support in messages
- **Xcode integration**: Create links to source files

### Phase 3: Global Hotkey
- Listen to Ctrl+Option+H globally
- Open menu bar or quick dialog
- Note: Requires accessibility permissions

### Phase 4: Extensions
- Finder extension for context menu
- Safari extension for quick access
- Spotlight integration for searching links

## File Structure Reference

| File | Purpose |
|------|---------|
| `HookmarksApp.swift` | @main, MenuBarExtra, Settings scene |
| `AppDelegate.swift` | URL scheme handler for `hook://` URIs |
| `MenuBarView.swift` | Menu bar UI with 3 tabs and controls |
| `HKBridge.swift` | Subprocess bridge to `hk` CLI |
| `FinderBridge.swift` | AppleScript interface to Finder/Safari |
| `PreferencesView.swift` | Settings window with 4 tabs |
| `Info.plist` | App bundle manifest with URL scheme |
| `Package.swift` | Swift Package manifest (macOS 13+) |

## Contributing

See the root `CONTRIBUTING.md` for guidelines. For the macOS app specifically:

1. Test in Xcode simulator or real hardware
2. Run `swift format` for code style
3. Add tests in `HookmarksTests/`
4. Update this README for new features
5. File an ADR for architectural decisions

## License

MIT. See root `LICENSE`.

---

**Status**: MVP Scaffold ✓ Ready for Xcode build, testing, and refinement.

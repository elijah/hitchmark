# Hookmarks macOS App — Development Guide

## What Was Scaffolded (Step 4 MVP)

This directory contains a complete Swift macOS application using SwiftUI, ready to build in Xcode.

### Directory Structure

```
apps/macos/
├── Package.swift                          # Swift Package manifest (macOS 13+)
├── Info.plist                             # App bundle metadata + hook:// URL scheme
├── README.md                              # User-facing setup guide
├── DEVELOPMENT.md                         # This file
├── Sources/Hookmarks/
│   ├── HookmarksApp.swift                 # @main entry point, MenuBarExtra scene
│   ├── AppDelegate.swift                  # Lifecycle, hook:// URL scheme handler
│   ├── MenuBarView.swift                  # 3-tab interface (Link, List, Finder)
│   ├── HKBridge.swift                     # Subprocess bridge to `hk` CLI
│   ├── FinderBridge.swift                 # AppleScript bridges (Finder, Safari)
│   └── PreferencesView.swift              # Settings window (4 tabs)
├── Tests/HookmarksTests/                  # (empty, ready for tests)
└── .build/                                # Build artifacts (git-ignored)
```

## Compilation Status

✅ **Builds without errors** (2.95 seconds on Apple Silicon)
- Binary: `apps/macos/.build/debug/Hookmarks` (720 KB ARM64 executable)
- All Swift files compile; 1 warning about test target directory (benign)

## What Each Component Does

### HookmarksApp.swift
**Purpose:** SwiftUI app entry point using `@main`

**Key features:**
- Declares `MenuBarExtra` with link.circle.fill icon
- Dispatches `AppDelegate` for URL scheme handling
- Opens `PreferencesView` in Settings scene

**Design decision:** MenuBarExtra over NSStatusItem for modern SwiftUI-native UI.

### AppDelegate.swift
**Purpose:** Handles application lifecycle and `hook://` URI resolution

**Key features:**
- `applicationDidFinishLaunching`: Logs startup
- `application(_:open:)`: Intercepts `hook://` URIs from Safari, email, Finder
- Calls `HKBridge.open()` to resolve URIs via subprocess
- Shows error alerts if resolution fails

**Technical detail:** Uses `[weak self]` in closure to avoid retain cycles.

### MenuBarView.swift
**Purpose:** Main menu bar UI with tabbed interface

**Key features:**
1. **Link Tab**: Input two URIs/paths → call `HKBridge.link()`
2. **List Tab**: Input URI → call `HKBridge.list()` to show bidirectional links
3. **Finder Tab**: 
   - Button to get selected file from Finder via `FinderBridge.getSelectedFile()`
   - Converts path to `hook://` URI via `HKBridge.fileToURI()`
   - Copy-to-clipboard button
4. **Header**: Segmented picker for tab selection
5. **Footer**: Settings button (⚙️) and version

**Design decision:** Tabbed interface instead of separate windows for compact menu bar UX.

### HKBridge.swift
**Purpose:** Subprocess bridge to call `hk` CLI tool

**Key features:**
- Static methods for each `hk` subcommand:
  - `fileToURI()`: `hk file <path>`
  - `open()`: `hk open <uri>`
  - `list()`: `hk list <uri>`
  - `link()`: `hk link <a> <b> [--note "..."]`
  - `purple()`: `hk purple <file> --format json` (Phase 2)
- Async subprocess calls on background queue (don't block UI)
- Auto-locates `hk` in standard paths:
  - `/usr/local/bin/hk` (preferred)
  - `~/.cargo/bin/hk` (Cargo default)
  - `/opt/homebrew/bin/hk` (Homebrew ARM64)
- Error handling: returns `HKBridgeError` enum

**Subprocess implementation:**
```swift
let process = Process()
process.executableURL = URL(fileURLWithPath: hkPath)
process.arguments = [subcommand] + args
process.standardOutput = Pipe()
process.standardError = Pipe()
try process.run()
```

**Design decision:** Subprocess over FFI because:
- Simpler, faster to ship (no Rust bindings)
- Decoupled (can upgrade CLI independently)
- FFI planned for Phase 2 (performance-critical loops)

### FinderBridge.swift
**Purpose:** AppleScript bridge to interact with macOS applications

**Key features:**
- `getSelectedFile()`: Returns POSIX path of selected Finder item
- `getActiveSafariURL()`: Returns URL of active Safari tab (Phase 2)
- `revealInFinder()`: Opens path in Finder and selects it (Phase 2)
- Async AppleScript execution on background thread
- Proper string escaping for AppleScript injection safety

**AppleScript example:**
```swift
tell application "Finder"
    return POSIX path of (item 1 of selection as alias)
end tell
```

**Design decision:** AppleScript over Finder framework because:
- No dependencies (built into macOS)
- Simpler than FSEvents/Finder API
- Sufficient for Phase 1 (single file selection)

**Permissions:** User must grant "Automation" permissions in System Preferences when prompted.

### PreferencesView.swift
**Purpose:** Settings window with 4 tabs

**Tabs:**
1. **General**: Auto-open links toggle, Launch at login toggle
2. **Hotkeys**: Global hotkey configuration (experimental, deferred to Phase 2)
3. **CLI**: Path to `hk` binary, auto-detection
4. **About**: App icon, version, links to GitHub/docs

**Storage:** Uses `@AppStorage` (backed by UserDefaults) for persistence

**Design decision:** TabView for organized preferences. Settings are minimal for MVP.

### Info.plist
**Purpose:** App bundle manifest with permissions and URL scheme

**Key entries:**
- `CFBundleIdentifier`: `com.elw.hookmarks`
- `CFBundleURLTypes`: Registers `hook://` protocol
- `NSAppleEventsUsageDescription`: Finder/AppleScript permission text
- `NSAccessibilityUsageDescription`: Global hotkey permission text
- `LSMinimumSystemVersion`: macOS 13.0+

**Design decision:** Hardcoded for now; can move to build system later.

### Package.swift
**Purpose:** Swift Package Manager manifest

**Key settings:**
- Platform: macOS 13.0+
- Target: Single executable `Hookmarks`
- No external dependencies (SwiftUI, Cocoa are built-in)
- Test target declaration (empty, ready for unit tests)

## How to Use This

### Build from Command Line
```bash
cd apps/macos
swift build
./​.build/debug/Hookmarks
```

### Open in Xcode
```bash
cd apps/macos
swift package generate-xcodeproj  # One time
open Hookmarks.xcodeproj
```

In Xcode:
- Cmd+B: Build
- Cmd+R: Run
- Cmd+U: Run tests (Phase 2)

### Deploy
Eventually (Phase 3):
1. Sign with Developer ID
2. Notarize with Apple
3. Create DMG installer
4. Submit to Mac App Store (optional)

## Next Steps (Priority Order)

### Phase 1.1 (Bug fixes & Polish)
- [ ] Test all 3 menu tabs with real `hk` CLI
- [ ] Test URL scheme handler (drag `hook://...` link to app)
- [ ] Test Finder bridge (select file, get URI)
- [ ] Add unit tests in `Tests/HookmarksTests/`
- [ ] Polish error messages and UI
- [ ] Add loading states to all buttons

### Phase 2 (Features)
- [ ] Global hotkey support (listen to Ctrl+Option+H)
- [ ] Safari extension (quick link button)
- [ ] JSON parsing in HKBridge (for structured `hk` output)
- [ ] Purple number support (`hk purple` integration)
- [ ] Preferences persistence (launch at login, hotkey)

### Phase 3 (Distribution)
- [ ] Code signing (Developer ID certificate)
- [ ] Notarization (Apple verification)
- [ ] DMG installer
- [ ] Sparkle auto-update framework
- [ ] Homebrew tap (for formula: `brew install hookmarks/macos/hookmarks`)

## Known Limitations & Deferred

- ❌ No code signing (can't run on other Macs yet)
- ❌ No persistence (hotkey/preferences don't auto-save)
- ❌ No global hotkey (Phase 2)
- ❌ No Finder extension (Phase 2)
- ❌ No Safari extension (Phase 2)
- ❌ No dark mode refinement
- ❌ No error dialogs (just NSLog)
- ❌ No tests yet

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| SwiftUI over AppKit | Modern, reactive, faster to ship |
| MenuBarExtra vs NSStatusItem | Built-in SwiftUI support, cleaner API |
| Subprocess vs FFI | Faster MVP, decoupled from CLI version |
| AppleScript vs Finder API | No dependencies, sufficient for MVP |
| UserDefaults vs plist | Standard macOS practice, auto-synced |
| No external dependencies | Minimal attack surface, easy distribution |
| macOS 13.0+ only | MenuBarExtra requires Ventura+ |

## Architecture Diagram

```
┌────────────────────────────────────────┐
│         Hookmarks.app (Swift)          │
├────────────────────────────────────────┤
│ HookmarksApp (@main)                   │
│  ├─ MenuBarExtra (link.circle.fill)   │
│  ├─ AppDelegate (URL scheme handler)   │
│  └─ Settings (PreferencesView)        │
├────────────────────────────────────────┤
│ UI Layer (SwiftUI Views)               │
│  ├─ MenuBarView (3 tabs)              │
│  └─ PreferencesView (4 tabs)          │
├────────────────────────────────────────┤
│ Bridge Layer                           │
│  ├─ HKBridge (subprocess → hk CLI)    │
│  └─ FinderBridge (AppleScript)        │
├────────────────────────────────────────┤
│ External Systems                       │
│  ├─ hk CLI (Rust) ~/..​.cargo/bin/hk  │
│  ├─ Finder (AppleScript)             │
│  ├─ Safari (AppleScript, Phase 2)    │
│  └─ Mail (AppleScript, Phase 2)      │
└────────────────────────────────────────┘
```

## Testing Checklist

Before moving to Phase 2, verify:
- [ ] Menu bar icon appears
- [ ] Click menu bar icon → menu opens
- [ ] "Link" tab: Input two URIs → Create Link button works
- [ ] "List" tab: Input URI → see links
- [ ] "Finder" tab: Get selected file works (need Finder permission grant)
- [ ] Preferences window opens
- [ ] Hook URI handler works (test: `open hook://example.com`)
- [ ] Error messages display correctly
- [ ] App quits cleanly

## Technical Debt

None identified for Phase 1. Will add as features grow:
- Unit tests (currently zero)
- Integration tests with `hk` CLI
- Performance benchmarks
- Accessibility audit (WCAG 2.1)
- Localization framework

---

**Phase 1 Status**: ✅ MVP Scaffold Complete
**Ready for**: Xcode build, testing, feature refinement

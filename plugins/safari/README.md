# Hookmarks — Safari Extension

A Safari Web Extension that lets you create and copy `hook://` links for any webpage via the [Hookmarks](../../README.md) `hk serve` HTTP API.

## Requirements

- macOS 12 (Monterey) or later
- Safari 15+
- Xcode 14+ (to build)
- [`hk serve`](../../crates/hookmarks-cli/) running locally on port 2701

## Project Structure

```
plugins/safari/
├── src/
│   ├── bridge.ts        # HTTP client for hk serve API
│   ├── background.ts    # Service worker — context menus, message bus
│   ├── popup.ts         # Toolbar popup controller
│   └── content.ts       # Content script (injected into pages)
├── Resources/           # Extension web assets (referenced by Xcode)
│   ├── manifest.json    # MV3 extension manifest
│   ├── popup.html       # Toolbar popup UI
│   ├── background.js    # Built output (from src/background.ts)
│   ├── popup.js         # Built output (from src/popup.ts)
│   └── content.js       # Built output (from src/content.ts)
├── esbuild.config.mjs   # Build script
├── package.json
└── tsconfig.json

apps/Hookmarks/          # Xcode wrapper project (generated)
├── Hookmarks.xcodeproj
├── Hookmarks/           # macOS host app (thin wrapper)
└── Hookmarks Extension/ # Safari extension target
```

## Development

### 1. Install dependencies

```sh
cd plugins/safari
npm install
```

### 2. Build the extension

```sh
npm run build          # development build (with sourcemaps)
npm run build:prod     # production build (minified)
```

Built JS files are emitted to `Resources/` and referenced directly by the Xcode project — **no copy step needed**.

### 3. Open in Xcode

```sh
open apps/Hookmarks/Hookmarks.xcodeproj
```

Press **⌘R** to build and run the host app.

### 4. Enable in Safari

1. Safari → **Settings** → **Advanced** → enable "Show features for web developers"
2. Safari → **Develop** → **Allow Unsigned Extensions**
3. Safari → **Settings** → **Extensions** → toggle **Hookmarks** on

The Hookmarks icon (🔗) will appear in the Safari toolbar.

### 5. Start the server

```sh
hk serve              # starts on http://127.0.0.1:2701
```

## Usage

| Action | How |
|--------|-----|
| Copy `hook://` URI for current page | Click toolbar icon → copy button |
| View links for current page | Click toolbar icon → links list |
| Create a link between two pages | Click toolbar icon → Add Link form |
| Copy URI via right-click | Right-click on page → "Copy hook:// link" |

The popup shows a **green "online" badge** when `hk serve` is reachable, or **red "offline"** when it isn't.

## Tests

```sh
npm test              # 11 Jest tests (bridge logic)
```

Tests cover `buildWebUri`, `probeServer` caching/invalidation, `createLink`, `listLinks`, and the server-not-running message.

## Architecture Notes

- **HTTP-only** — browser extensions cannot spawn subprocesses. All operations require `hk serve` to be running.
- **`buildWebUri(url)`** — encodes a web page URL as URL-safe base64 (no `+`, `/`, or `=` padding) and wraps it as `hook://file/<encoded>`. This is deterministic and reversible.
- **Background service worker** — handles context menu clicks and relays messages from the popup. Safari MV3 service workers are short-lived; state is persisted via `chrome.storage.local`.
- **Server URL** — defaults to `http://127.0.0.1:2701`; configurable via the Settings section in the popup (persisted to extension storage).

## Distribution

To distribute to other Macs, the app must be signed with an Apple Developer ID:

1. In Xcode, select the **Hookmarks** target → Signing & Capabilities → set your Team
2. Archive → Distribute App → Developer ID
3. Notarize via `xcrun notarytool`

For personal/team use, unsigned extensions with "Allow Unsigned Extensions" enabled is sufficient.

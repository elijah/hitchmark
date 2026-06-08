# Hookmarks — Web Extension (Safari / Chrome / Edge / Brave)

A Web Extension (Manifest V3) that lets you create and copy `hook://` links for any webpage via the [Hookmarks](../../README.md) `hk serve` HTTP API.

**One codebase — runs in Safari, Chrome, Edge, Brave, Arc, and any Chromium-based browser.**

## Requirements

- [`hk serve`](../../crates/hitchmark-cli/) running locally on port 2701
- **Safari**: macOS 12+, Safari 15+, Xcode 14+ (to build the wrapper)
- **Chrome / Brave / Arc**: Chrome 88+ (any platform — macOS, Windows, Linux)
- **Edge**: Edge 88+ (any platform)

## Project Structure

```
plugins/safari/
├── src/
│   ├── bridge.ts        # HTTP client for hk serve API
│   ├── background.ts    # Service worker — context menus, message bus
│   ├── popup.ts         # Toolbar popup controller
│   └── content.ts       # Content script (injected into pages)
├── Resources/           # Extension web assets — loaded by all browsers
│   ├── manifest.json    # MV3 extension manifest
│   ├── popup.html       # Toolbar popup UI
│   ├── background.js    # Built output
│   ├── popup.js         # Built output
│   └── content.js       # Built output
├── scripts/
│   └── package-chrome.mjs  # Packages Resources/ as .zip for store upload
├── esbuild.config.mjs   # Build script
├── package.json
└── tsconfig.json

apps/Hookmarks/          # Xcode wrapper for Safari (generated, macOS only)
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
npm run build:prod     # production build (minified, for packaging)
```

Built JS files emit to `Resources/`. The Safari Xcode project references them directly — no copy step.

---

## Installing in Chrome / Brave / Arc

**Option A — Load unpacked (development)**

1. `npm run build`
2. Open `chrome://extensions` (or `brave://extensions`, `arc://extensions`)
3. Enable **Developer mode** (top-right toggle)
4. Click **Load unpacked** → select `plugins/safari/Resources/`
5. The Hookmarks icon appears in your toolbar

**Option B — Packaged `.zip` (distribution / CRX)**

```sh
npm run package:chrome     # → hookmarks-chrome-0.2.0.zip
```

Upload the zip to the [Chrome Web Store Developer Dashboard](https://chrome.google.com/webstore/devconsole).

---

## Installing in Edge

**Option A — Load unpacked**

1. `npm run build`
2. Open `edge://extensions`
3. Enable **Developer mode**
4. Click **Load unpacked** → select `plugins/safari/Resources/`

**Option B — Submit to Edge Add-ons store**

```sh
npm run package:edge       # → hookmarks-edge-0.2.0.zip
```

Upload to [Microsoft Partner Center](https://partner.microsoft.com/dashboard/microsoftedge).

---

## Installing in Safari (macOS)

### Build the Xcode wrapper

```sh
open apps/Hookmarks/Hookmarks.xcodeproj
```

Press **⌘R** to build and run the host app.

### Enable in Safari

1. Safari → **Settings** → **Advanced** → enable "Show features for web developers"
2. Safari → **Develop** → **Allow Unsigned Extensions**
3. Safari → **Settings** → **Extensions** → toggle **Hookmarks** on

---

## Usage

| Action | How |
|--------|-----|
| Copy `hook://` URI for current page | Click toolbar icon → copy button |
| View links for current page | Click toolbar icon → links list |
| Create a link between two pages | Click toolbar icon → Add Link form |
| Copy URI via right-click | Right-click on page → "Copy hook:// link" |
| Change server URL | Click toolbar icon → ⚙ Settings |

The popup shows a **green "online" badge** when `hk serve` is reachable.

### Start the server

```sh
hk serve              # starts on http://127.0.0.1:2701
```

---

## Tests

```sh
npm test              # 11 Jest tests (bridge logic)
```

Tests cover `buildWebUri`, `probeServer` caching/invalidation, `createLink`, `listLinks`, and the server-not-running message.

---

## Architecture Notes

- **HTTP-only** — browser extensions cannot spawn subprocesses; `hk serve` must be running.
- **`buildWebUri(url)`** — encodes a web URL as URL-safe base64 (no `+`, `/`, `=`) → `hook://file/<encoded>`. Deterministic and reversible.
- **Manifest V3** — service worker background, `contextMenus` + `storage` + `scripting` + `activeTab` permissions.
- **Same code, all browsers** — the MV3 `chrome.*` API is implemented by Chrome, Edge, Brave, Arc, and (via the Xcode wrapper) Safari. No browser-specific forks.
- **Server URL** — defaults to `http://127.0.0.1:2701`; configurable in the popup Settings section (persisted via `chrome.storage.local`).

---

## Distribution

| Browser | Store | Notes |
|---------|-------|-------|
| Chrome | [Chrome Web Store](https://chrome.google.com/webstore/devconsole) | Upload `hookmarks-chrome-<v>.zip` |
| Edge | [Edge Add-ons](https://partner.microsoft.com/dashboard/microsoftedge) | Upload `hookmarks-edge-<v>.zip` (same zip) |
| Brave / Arc | Sideload via `chrome://extensions` | No store submission needed |
| Safari | [Mac App Store](https://appstoreconnect.apple.com) | Requires Apple Developer ID + notarization |

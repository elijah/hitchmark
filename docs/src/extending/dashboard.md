# Web Dashboard

`hk serve` includes a built-in web dashboard served at `http://127.0.0.1:2701/`.

## Starting the dashboard

```bash
hk serve
# 🔗 Hitchmark server listening on http://127.0.0.1:2701
```

Then open [http://127.0.0.1:2701](http://127.0.0.1:2701) in your browser.

## Features

| Feature | Description |
|---------|-------------|
| **Links table** | All links in the store, with copy-URI buttons |
| **Bookmarks table** | All bookmarks with file paths |
| **Search** | Per-section text filter |
| **Sort** | Click any column header to sort |
| **Auto-refresh** | Polls every 30 seconds for changes |
| **Dark/light mode** | Follows `prefers-color-scheme` |
| **Responsive** | Adapts to narrow viewports |

## API endpoints

The dashboard uses these JSON endpoints (also usable directly):

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | Dashboard HTML |
| `GET` | `/health` | `{ "status": "ok", "version": "..." }` |
| `GET` | `/links?uri=<uri>` | Links for a specific resource |
| `GET` | `/links/all` | All links in the store |
| `POST` | `/links` | Create a link (`{ uri_a, uri_b, note? }`) |
| `DELETE` | `/links` | Delete a link (`{ uri_a, uri_b }`) |
| `GET` | `/bookmarks` | All bookmarks |
| `GET` | `/uri?path=<path>` | Convert file path → `hook://` URI |
| `GET` | `/purple?path=<path>` | Purple number IDs for a file |

## Using from scripts

```bash
# List all links as JSON
curl http://127.0.0.1:2701/links/all | jq .

# Get URI for a file
curl "http://127.0.0.1:2701/uri?path=$(pwd)/README.md"

# Create a link
curl -X POST http://127.0.0.1:2701/links \
  -H 'Content-Type: application/json' \
  -d '{"uri_a":"hook://file/...","uri_b":"hook://file/..."}'
```

## Custom port

```bash
hk serve --port 3000
# 🔗 Hitchmark server listening on http://127.0.0.1:3000
```

## Auto-start

- **macOS**: launchd LaunchAgent — `apps/macos/launchd/app.hitchmark.serve.plist`
- **Linux**: systemd --user — `apps/linux-tray/hitchmark-serve.service`
- **Windows tray**: enable `auto_start_server = true` in `%APPDATA%\hitchmark\tray.toml`

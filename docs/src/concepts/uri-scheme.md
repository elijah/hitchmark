# URI Scheme Reference

The `hook://` URI scheme provides stable, addressable links to any resource.

## Grammar

```
hook-uri     = "hook://" authority "/" path ["#" fragment]
authority    = "file" | "bookmark" | "x-callback-url"

; File URIs
file-uri     = "hook://file/" base64url-path ["#" para-id]
base64url-path = *( ALPHA / DIGIT / "-" / "_" )  ; Base64url, no padding
para-id      = "para-" purple-id
purple-id    = 6*8( base58-char )
base58-char  = "1"-"9" / "A"-"H" / "J"-"N" / "P"-"Z" / "a"-"k" / "m"-"z"

; Bookmark URIs
bookmark-uri = "hook://bookmark/" uuid
uuid         = 8HEXDIG "-" 4HEXDIG "-" 4HEXDIG "-" 4HEXDIG "-" 12HEXDIG

; x-callback-url (app-to-app)
callback-uri = "hook://x-callback-url/" action
action       = *( UNRESERVED / pct-encoded )
```

## URI types

### File URI

Links to a file, with an optional paragraph address.

```
hook://file/L1VzZXJzL2Vsd...
hook://file/L1VzZXJzL2Vsd...#para-7nxxnx
```

The path component is the **base64url-encoded absolute path** (RFC 4648,
no padding). This encoding:
- Handles spaces and special characters
- Is safe in URLs and HTML attributes
- Survives copy-paste across systems

**Decoding example:**
```
hook://file/L1VzZXJzL2Vsd...
              ↓ base64url decode
              /Users/elw/docs/project.md
```

**Python:**
```python
import base64
path = base64.urlsafe_b64decode(encoded + "==").decode()
```

**JavaScript:**
```js
const path = atob(encoded.replace(/-/g, '+').replace(/_/g, '/'));
```

**Rust:**
```rust
use base64::{engine::general_purpose, Engine};
let path = general_purpose::URL_SAFE_NO_PAD.decode(encoded)?;
```

### Bookmark URI

Links to a non-file resource (web URL, email, etc.) identified by a UUID.

```
hook://bookmark/550e8400-e29b-41d4-a716-446655440000
```

Bookmark metadata (title, target URL, tags) is stored in the link database.
Resolving a bookmark URI looks up the metadata and opens the target URL.

> **Status:** Bookmark creation and resolution are not yet implemented in v0.1.
> File URIs are fully supported.

### x-callback-url

App-to-app communication following the
[x-callback-url](http://x-callback-url.com/) convention.

```
hook://x-callback-url/create-link?source=...&target=...
```

> **Status:** Deferred to v0.2 for legacy Hook.app compatibility.

## Fragment: paragraph address

A `#para-<id>` fragment targets a specific paragraph within a document.

```
hook://file/L1VzZXJzL2Vsd...#para-7nxxnx
                               └─ Purple number (6 or 8 chars, base58)
```

See [Purple Numbers](./purple-numbers.md) for the ID generation algorithm.

## Normalization

Before storing or comparing URIs, apply these normalization steps:

1. Scheme is lowercase: `hook://` not `HOOK://`
2. The authority component (`file`, `bookmark`) is lowercase
3. The path is base64url without padding (no trailing `=`)
4. Fragment, if present, uses the `para-` prefix
5. Trailing slashes are stripped

Two URIs are equal if and only if their normalized forms are identical.

## Security & privacy

- `hook://` URIs contain file paths (via base64). **Do not share them
  publicly** if the path reveals sensitive information (e.g., username,
  project name).
- The link database (`store.db`) is local and never synced automatically.
- No network requests are made when resolving file URIs.

## Resolution algorithm

```
resolve(uri):
  1. Parse uri into (type, payload, fragment)
  2. If type == "file":
       a. Decode base64url payload → absolute path
       b. If path does not exist → error "File not found"
       c. If fragment present → open file + scroll to anchor
       d. Else → open file with system default app
  3. If type == "bookmark":
       a. Look up UUID in store.db → get target_url
       b. Open target_url with system browser
  4. If type == "x-callback-url":
       a. Invoke callback (not yet implemented)
```

## Examples

```bash
# Get the hook:// URI for a file
hk file ~/docs/meeting-notes.md
# → hook://file/L1VzZXJzL2Vsd...

# Open a file URI
hk open "hook://file/L1VzZXJzL2Vsd..."

# Open to a specific paragraph
hk open "hook://file/L1VzZXJzL2Vsd...#para-7nxxnx"

# Create a link between two files
hk link ~/docs/project.md ~/docs/reference.md
```

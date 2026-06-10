# Hitchmark URI Scheme Specification

**Version:** 0.3  
**Status:** Stable  
**Last Updated:** 2026-06-10

## Overview

The `hook://` URI scheme provides stable, addressable links to files, web pages, emails, and intra-document locations. Unlike `http://` URIs (which may break when servers move), `hook://` URIs are rooted in local storage or stable identifiers, surviving document reorganization and minor content changes.

## Grammar

```
hook-uri = "hook://" hook-body ["#" fragment]

hook-body = file-uri
          | bookmark-uri
          | callback-uri

file-uri = "file/" base64url-path
bookmark-uri = "bookmark/" uuid
callback-uri = "x-callback-url/" action

base64url-path = base64url-encoded (absolute filesystem path, UTF-8, percent-encoded)
uuid = uuid-v4-or-similar
action = 1*pchar (URL-encoded path, e.g., "open?url=...")
fragment = para-id / other-fragment

para-id = "para-" base58-short
base58-short = 6-8 base58-chars
```

## URI Types

### 1. File URI

**Purpose:** Address a file on the local filesystem, with optional paragraph reference.

**Syntax:**
```
hook://file/<base64url-absolute-path>[#para-<id>]
```

**Examples:**
```
hook://file/L2hvbWUvYWxpY2UvZG9jcy9ub3Rlcy50eHQ
hook://file/L3Zhci9sb2cvYXBwLmxvZw#para-abc123
hook://file/L1VzZXJzL2pvZS9PYnNpZGlhbi9WYXVsdC9EYWlseS50eHQ#para-xyz789
```

**Semantics:**
- The path is always **absolute** (starts with `/`); relative paths MUST be expanded before encoding
- Encoding: standard base64url (RFC 4648 §5), **no padding** (trailing `=` characters)
- Non-ASCII path characters MUST be percent-encoded before base64 encoding
- The fragment `#para-<id>` is optional; see § Purple Numbers for syntax and semantics
- The file need not exist at URI creation time; resolution can fail gracefully
- On Windows, paths are converted to forward-slash format (e.g., `C:\Users\...` → `C:/Users/...`) before base64 encoding

### 2. Bookmark URI

**Purpose:** Reference a file by a stable UUID that survives renames and moves.

**Syntax:**
```
hook://bookmark/<uuid>
```

**Examples:**
```
hook://bookmark/550e8400-e29b-41d4-a716-446655440000
hook://bookmark/f47ac10b-58cc-4372-a567-0e02b2c3d479
```

**Semantics:**
- The UUID is typically v4 (random) but MAY be v5 (derived from namespace + name)
- The actual file path is stored in a **link metadata store**; the URI is only a stable reference
- When a bookmarked file is moved or renamed, update the store via `hk bookmark update`
- Bookmark URIs **never contain the resource directly**; this preserves stability when the underlying file moves
- Hitchmark's `hk watch` command automatically updates bookmark paths when files are renamed

### 3. x-callback-url (Interop)

**Purpose:** App-to-app callbacks; compatibility with the original Hook.app protocol.

**Syntax:**
```
hook://x-callback-url/<action>[?<params>]
```

**Parameters (common):**

| Parameter | Description |
|-----------|-------------|
| `source` | Source hook:// URI (percent-encoded) |
| `target` | Target hook:// URI (percent-encoded) |
| `x-success` | Callback URI on success |
| `x-error` | Callback URI on error |

**Example:**
```
hook://x-callback-url/create-link?source=hook%3A%2F%2Ffile%2F...&target=hook%3A%2F%2Fbookmark%2F...
```

**Status:** Applications MAY handle these URIs; unknown actions SHOULD be silently ignored.

## Fragment Semantics

### Purple Numbers (`#para-<id>`)

See § Purple Numbers specification.

### Other Fragments

Fragments other than `para-*` MAY be used for application-specific purposes (e.g., `#section-intro`, `#line-42`) but are not standardized. Applications SHOULD preserve unknown fragments when resolving and re-serializing URIs.

## URI Validation

A hook:// URI is **valid** if:

1. It starts with `hook://`
2. The body matches one of the three URI types (file, bookmark, callback)
3. For file URIs: the base64url string decodes to valid UTF-8 and represents a valid filesystem path
4. For bookmark URIs: the UUID is valid per RFC 4122
5. The fragment (if present) is either `para-<6-8 base58 chars>` or an unrecognized fragment (allowed for extensibility)

## URI Normalization

Implementations MUST normalize URIs as follows:

1. **Whitespace:** Strip leading/trailing whitespace before parsing
2. **Scheme:** `hook://` (lowercase; `HOOK://` is invalid)
3. **Path separators:** On Windows, normalize to forward slashes in the encoded path
4. **Base64:** Apply URL-safe decoding with no padding; re-encode consistently with no padding
5. **Fragment:** Lowercase the fragment name (e.g., `#para-ABC` → `#para-abc`)

Two URIs are **equal** if their normalized forms are byte-for-byte identical.

## Link Metadata Schema

When creating a link between two URIs, implementations MUST record:

```json
{
  "source_uri": "hook://file/...",
  "target_uri": "hook://bookmark/...",
  "created_at": "2026-06-10T13:12:00Z",
  "note": "optional human-readable annotation",
  "tags": ["optional", "labels"],
  "bidirectional": true
}
```

**Fields:**
- `source_uri`, `target_uri`: Full hook:// URIs (normalized)
- `created_at`: RFC 3339 timestamp (UTC preferred)
- `note`: Optional; max 500 characters; no special formatting required
- `tags`: Optional array of tags; implementations MAY use these for organizing links
- `bidirectional`: If `true`, the link is symmetric; both `source → target` and `target → source` are valid queries

## Content Identity

### Stability Guarantees

A `hook://file/<path>` URI points to a specific **file path**, not file content. The file can change, move, be deleted, or be recreated without invalidating the URI.

The optional `#para-<id>` fragment points to a **specific paragraph by content hash**, not by line number. The paragraph can move within the document, but the ID remains valid as long as the content is stable (see § Purple Numbers).

### Path-Based vs. Content-Based Addressing

This spec uses **path-based addressing** for files:
- Pro: Simple, immediate; no need to scan file content
- Con: Links break if the file is moved or renamed (mitigated by `hook://bookmark/` URIs and `hk watch`)

Future versions MAY add content-hash-based addressing (e.g., `hook://file-content/<sha256-hash>`) for stronger stability, but this is beyond scope for v0.3.

## Resolution Algorithm

To resolve a `hook://` URI:

1. Parse the URI according to § Grammar
2. If file URI:
   a. Decode the base64url path
   b. Check if the file exists; if not, fail gracefully (user-facing error: "File not found")
   c. If fragment is present, seek the paragraph by ID within the file (see § Purple Numbers)
   d. Open/display the file (and highlight the paragraph if found)
3. If bookmark URI:
   a. Query the link store for the file path associated with this UUID
   b. If path exists, open the file; otherwise report "Bookmark target not found"
4. If callback URI:
   a. Route to registered handler (typically the creating application)
   b. Handlers are defined outside this spec (implementation detail)

## Examples

### Creating a link to a file

User creates a link from their note to a shared document:

```
Note ID: hook://bookmark/123e4567-e89b-12d3-a456-426614174000
File: hook://file/L1VzZXJzL2Fiam9uL0RvY3MvcHJvamVjdC1wcm9wb3NhbC5wZGY
```

The link metadata store records:
```json
{
  "source_uri": "hook://bookmark/123e4567-e89b-12d3-a456-426614174000",
  "target_uri": "hook://file/L1VzZXJzL2Fiam9uL0RvY3MvcHJvamVjdC1wcm9wb3NhbC5wZGY",
  "created_at": "2026-06-10T14:30:00Z",
  "note": "Shared with team for Q3 planning",
  "bidirectional": true
}
```

### Linking to a specific paragraph

User creates a link from a paragraph in their note to a paragraph in a shared document:

```
Source: hook://bookmark/note-uuid#para-abc123
Target: hook://file/L1VzZXJzL2Fiam9uL0RvY3MvcHJvamVjdC1wcm9wb3NhbC5wZGY#para-xyz789
```

Both the source and target are now precisely addressable by paragraph ID.

## Security Considerations

### Trust Model

The `hook://` URI scheme assumes:
- **Local trust:** If a file is readable by your user, any URI can reference it
- **No authentication:** URIs are not cryptographically signed; anyone can create a URI
- **No authorization:** The scheme enforces no access control; the OS filesystem is the boundary

### Privacy

- File URIs contain absolute paths; sharing a URI may reveal system structure
- Bookmark URIs are opaque identifiers; they don't leak information
- Link metadata is stored locally; sharing a URI reveals only that identifier, not the link graph

### Injection

Applications MUST:
- Validate all URIs before opening files
- Respect OS file permissions (don't use elevated privileges to bypass security)
- Warn if a URI is from an untrusted source (e.g., pasted from email)

## Versioning

This spec is at **v0.3**. Future versions MAY:
- Add content-hash-based file addressing for stability across renames
- Add fragment types for table cells, code ranges, or other structured content
- Add cryptographic signing for trusted link chains
- Add query parameters for link-time options (e.g., `?mode=append` to append text to a link note)

Implementations SHOULD gracefully ignore unknown URI types and fragments.

## Changelog

### v0.3 (2026-06-10)

- Renamed project to Hitchmark
- Clarified bookmark URI semantics (file-path-based, not arbitrary resource)
- Added `hk watch` auto-repair note to bookmark section
- Added x-callback-url parameter table and percent-encoding note
- Added `HK_STORE_PATH` env var mention in resolution algorithm context
- Updated examples with current date

### v0.2 (2025-06-06)

- Added bookmark URI section
- Added x-callback-url interop section
- Added resolution algorithm

### v0.1 (2025-06-06)

- Initial release
- Three URI types: file, bookmark, callback
- Base64url encoding for file paths
- Purple number fragments
- Link metadata schema


## Overview

The `hook://` URI scheme provides stable, addressable links to files, web pages, emails, and intra-document locations. Unlike `http://` URIs (which may break when servers move), `hook://` URIs are rooted in local storage or stable identifiers, surviving document reorganization and minor content changes.

## Grammar

```
hook-uri = "hook://" hook-body ["#" fragment]

hook-body = file-uri
          | bookmark-uri
          | callback-uri

file-uri = "file/" base64url-path
bookmark-uri = "bookmark/" uuid
callback-uri = "x-callback-url/" action

base64url-path = base64url-encoded (absolute filesystem path)
uuid = uuid-v4-or-similar
action = 1*pchar (URL-encoded path, e.g., "open?url=...")
fragment = para-id / other-fragment

para-id = "para-" base58-short
base58-short = 6-8 base58-chars
```

## URI Types

### 1. File URI

**Purpose:** Address a file on the local filesystem, with optional paragraph reference.

**Syntax:**
```
hook://file/<base64url-absolute-path>[#para-<id>]
```

**Examples:**
```
hook://file/L2hvbWUvYWxpY2UvZG9jcy9ub3Rlcy50eHQ
hook://file/L3Zhci9sb2cvYXBwLmxvZw#para-abc123
hook://file/L1VzZXJzL2pvZS9PYnNpZGlhbi9WYXVsdC9EYWlseS50eHQ#para-xyz789
```

**Semantics:**
- The path is always **absolute** (starts with `/`); relative paths MUST be expanded before encoding
- Encoding: standard base64url (RFC 4648 §5), **no padding** (trailing `=` characters)
- The fragment `#para-<id>` is optional; see § Purple Numbers for syntax and semantics
- The file need not exist at URI creation time; resolution can fail gracefully
- On Windows, paths are converted to forward-slash format (e.g., `C:\Users\...` → `C:/Users/...`) before base64 encoding

### 2. Bookmark URI

**Purpose:** Reference non-file resources: web pages, emails, or application-specific objects.

**Syntax:**
```
hook://bookmark/<uuid>
```

**Examples:**
```
hook://bookmark/550e8400-e29b-41d4-a716-446655440000
hook://bookmark/f47ac10b-58cc-4372-a567-0e02b2c3d479
```

**Semantics:**
- The UUID is typically v4 (random) but MAY be v5 (derived from namespace + name)
- The actual resource (URL, email, note ID) is stored in a **link metadata store**; the URI is only a stable reference
- Bookmark URIs **never contain the resource directly**; this preserves privacy and stability when the underlying resource moves

### 3. x-callback-url (Legacy/Interop)

**Purpose:** Support app-to-app callbacks; reserved for compatibility with the original Hookmarks protocol.

**Syntax:**
```
hook://x-callback-url/<action>[?<params>]
```

**Example:**
```
hook://x-callback-url/create-link?source=...&target=...&callback=...
```

**Status:** Not required for v0.1. Applications MAY ignore these URIs.

## Fragment Semantics

### Purple Numbers (`#para-<id>`)

See § Purple Numbers specification.

### Other Fragments

Fragments other than `para-*` MAY be used for application-specific purposes (e.g., `#section-intro`, `#line-42`) but are not standardized. Applications SHOULD preserve unknown fragments when resolving and re-serializing URIs.

## URI Validation

A hook:// URI is **valid** if:

1. It starts with `hook://`
2. The body matches one of the three URI types (file, bookmark, callback)
3. For file URIs: the base64url string decodes to valid UTF-8 and represents a valid filesystem path
4. For bookmark URIs: the UUID is valid per RFC 4122
5. The fragment (if present) is either `para-<6-8 base58 chars>` or an unrecognized fragment (allowed for extensibility)

## URI Normalization

Implementations MUST normalize URIs as follows:

1. **Whitespace:** Strip leading/trailing whitespace before parsing
2. **Scheme:** `hook://` (lowercase; `HOOK://` is invalid)
3. **Path separators:** On Windows, normalize to forward slashes in the encoded path
4. **Base64:** Apply URL-safe decoding with no padding; re-encode consistently with no padding
5. **Fragment:** Lowercase the fragment name (e.g., `#para-ABC` → `#para-abc`)

Two URIs are **equal** if their normalized forms are byte-for-byte identical.

## Link Metadata Schema

When creating a link between two URIs, implementations MUST record:

```json
{
  "source_uri": "hook://file/...",
  "target_uri": "hook://bookmark/...",
  "created_at": "2025-06-06T13:12:00Z",
  "note": "optional human-readable annotation",
  "tags": ["optional", "labels"],
  "bidirectional": true
}
```

**Fields:**
- `source_uri`, `target_uri`: Full hook:// URIs (normalized)
- `created_at`: RFC 3339 timestamp (UTC preferred)
- `note`: Optional; max 500 characters; no special formatting required
- `tags`: Optional array of tags; implementations MAY use these for organizing links
- `bidirectional`: If `true`, the link is symmetric; both `source → target` and `target → source` are valid queries

## Content Identity

### Stability Guarantees

A `hook://file/<path>` URI points to a specific **file path**, not file content. The file can change, move, be deleted, or be recreated without invalidating the URI.

The optional `#para-<id>` fragment points to a **specific paragraph by content hash**, not by line number. The paragraph can move within the document, but the ID remains valid as long as the content is stable (see § Purple Numbers).

### Path-Based vs. Content-Based Addressing

This spec uses **path-based addressing** for files:
- Pro: Simple, immediate; no need to scan file content
- Con: Links break if the file is moved or renamed

Future versions MAY add content-hash-based addressing (e.g., `hook://file-content/<sha256-hash>`) for stronger stability, but this is beyond scope for v0.1.

## Resolution Algorithm

To resolve a `hook://` URI:

1. Parse the URI according to § Grammar
2. If file URI:
   a. Decode the base64url path
   b. Check if the file exists; if not, fail gracefully (user-facing error: "File not found")
   c. If fragment is present, seek the paragraph by ID within the file (see § Purple Numbers)
   d. Open/display the file (and highlight the paragraph if found)
3. If bookmark URI:
   a. Query the link store for metadata associated with this UUID
   b. Retrieve the target resource (URL, email, note)
   c. Open/display the resource
4. If callback URI:
   a. Route to registered handler (typically the creating application)
   b. Handlers are defined outside this spec (implementation detail)

## Examples

### Creating a link to a file

User creates a link from their note to a shared document:

```
Note ID: hook://bookmark/123e4567-e89b-12d3-a456-426614174000
File: hook://file/L1VzZXJzL2Fiam9uL0RvY3MvcHJvamVjdC1wcm9wb3NhbC5wZGY
```

The link metadata store records:
```json
{
  "source_uri": "hook://bookmark/123e4567-e89b-12d3-a456-426614174000",
  "target_uri": "hook://file/L1VzZXJzL2Fiam9uL0RvY3MvcHJvamVjdC1wcm9wb3NhbC5wZGY",
  "created_at": "2025-06-06T14:30:00Z",
  "note": "Shared with team for Q3 planning",
  "bidirectional": true
}
```

### Linking to a specific paragraph

User creates a link from a paragraph in their note to a paragraph in a shared document:

```
Source: hook://bookmark/note-uuid#para-abc123
Target: hook://file/L1VzZXJzL2Fiam9uL0RvY3MvcHJvamVjdC1wcm9wb3NhbC5wZGY#para-xyz789
```

Both the source and target are now precisely addressable by paragraph ID.

## Security Considerations

### Trust Model

The `hook://` URI scheme assumes:
- **Local trust:** If a file is readable by your user, any URI can reference it
- **No authentication:** URIs are not cryptographically signed; anyone can create a URI
- **No authorization:** The scheme enforces no access control; the OS filesystem is the boundary

### Privacy

- File URIs contain absolute paths; sharing a URI may reveal system structure
- Bookmark URIs are opaque identifiers; they don't leak information
- Link metadata is stored locally; sharing a URI reveals only that identifier, not the link graph

### Injection

Applications MUST:
- Validate all URIs before opening files
- Respect OS file permissions (don't use elevated privileges to bypass security)
- Warn if a URI is from an untrusted source (e.g., pasted from email)

## Versioning

This spec is at **v0.1**. Future versions MAY:
- Add content-hash-based file addressing for stability across renames
- Add fragment types for table cells, code ranges, or other structured content
- Add cryptographic signing for trusted link chains
- Add query parameters for link-time options (e.g., `?mode=append` to append text to a link note)

Implementations SHOULD gracefully ignore unknown URI types and fragments.

## Changelog

### v0.1 (2025-06-06)

- Initial release
- Three URI types: file, bookmark, callback
- Base64url encoding for file paths
- Purple number fragments
- Link metadata schema

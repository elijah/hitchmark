# Purple Numbers

Purple numbers are **stable, human-readable IDs for individual paragraphs**.
They let you link to (and permanently address) any paragraph in any document.

The name comes from Doug Engelbart's original vision of
[purple numbers](https://en.wikipedia.org/wiki/PurpleNumbers) as permanent
paragraph addresses in the web.

## What makes a purple number stable?

A purple number is computed from the **content** of the paragraph — not its
position. This means:

- ✅ Paragraphs can be **reordered** without changing their IDs
- ✅ Minor edits (< 50% Levenshtein distance) preserve the ID
- ✅ IDs survive file renames and moves
- ❌ Major rewrites (> 50% change) produce a new ID
- ❌ Identical paragraphs in the same document get the same base ID
  (handled by extending to 8 chars on collision)

## Algorithm

The algorithm is defined in [`specs/purple-numbers.md`](../../../specs/purple-numbers.md)
and implemented identically in Rust (`hitchmark-core`) and TypeScript
(`plugins/obsidian`).

```
purple_id(paragraph_text):
  1. Encode text as UTF-8 bytes
  2. Compute SHA-256(bytes) → 32-byte hash
  3. Encode as Base58 (Bitcoin alphabet, no padding)
  4. Take first 6 characters
  5. If collision within document: take first 8 characters
```

**Base58 alphabet** (same as Bitcoin, bs58 crate):
```
123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz
```
*(Excludes: 0, O, I, l — characters that look alike)*

## Example

```python
import hashlib, base58

text = "Hello world"
digest = hashlib.sha256(text.encode()).digest()
encoded = base58.b58encode(digest).decode()
purple_id = encoded[:6]
# → "7nxxnx"
```

Verified cross-language:
```
Rust (hitchmark-core):  "Hello world" → 7nxxnx ✅
TypeScript (plugin):    "Hello world" → 7nxxnx ✅
```

## Paragraph definition

A "paragraph" is defined as any block of text separated by one or more blank
lines. In Markdown:

```markdown
This is paragraph one.
Still paragraph one (single newline).

This is paragraph two (blank line above).

This is paragraph three.
```

Code blocks, headings, lists, and blockquotes are also treated as paragraphs
(each is a standalone addressable block).

## Collision handling

If two paragraphs in the same document produce the same 6-character ID:

1. The first paragraph keeps the 6-character ID
2. The colliding paragraph gets an 8-character ID

This is handled automatically by `PurpleNumberGenerator` (Rust) and
`computeDocumentPurpleIds` (TypeScript).

## URI fragment format

Paragraph addresses are appended to `hook://` file URIs as fragments:

```
hook://file/<base64-path>#para-<purple-id>
```

Examples:
```
hook://file/L1VzZXJzL2Vsd...#para-7nxxnx
hook://file/L1VzZXJzL2Vsd...#para-3qrstu
```

## Rendering

In the Obsidian plugin, purple numbers are rendered as small superscript
annotations in the live editor margin:

```
This is a paragraph. §7nxxnx
```

The annotation uses CSS class `.hookmarks-purple-number`:

```css
.hookmarks-purple-number {
  color: var(--hookmarks-purple-color, #888);
  font-size: 0.7em;
  font-family: var(--font-monospace);
  vertical-align: super;
  cursor: pointer;
  opacity: 0.7;
}
```

The color is configurable in Obsidian Settings → Hookmarks.

## Generating purple numbers from the CLI

```bash
# Markdown output
hk purple ~/docs/note.md

# JSON output
hk purple ~/docs/note.md --format json
```

JSON output example:
```json
[
  {
    "id": "7nxxnx",
    "text": "Hello world",
    "uri_fragment": "para-7nxxnx"
  }
]
```

## Accessibility

Purple numbers in the Obsidian plugin include ARIA labels:

```html
<span
  class="hookmarks-purple-number"
  role="button"
  tabindex="0"
  aria-label="Purple number 7nxxnx — click to copy hook URI"
>§7nxxnx</span>
```

Keyboard users can navigate to any purple number with Tab and activate it
with Enter or Space to copy the URI.

## Stability guarantee

An ID computed for paragraph text `T` is **stable** as long as the
edit distance from `T` to the current text is ≤ 50% of `T`'s length
(Levenshtein distance).

| Change type | ID preserved? |
|-------------|---------------|
| Reorder paragraphs | ✅ Yes |
| Fix a typo (< 50% change) | ✅ Yes |
| Rename the file | ✅ Yes |
| Move the file | ✅ Yes |
| Rewrite the paragraph (> 50% change) | ❌ No |
| Delete and recreate the paragraph | ❌ No |

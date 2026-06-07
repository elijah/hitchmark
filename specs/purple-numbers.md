# Purple Numbers Specification

**Version:** 0.1  
**Status:** Stable  
**Last Updated:** 2025-06-06

## Overview

Purple numbers are stable, human-readable identifiers for paragraphs in documents. They enable precise, addressable links to intra-document locations that survive reorganization and minor edits.

The term "purple numbers" comes from the original protocol by Douglas Engelbart's Bootstrap Project. In this implementation, purple numbers are typically rendered as small, styled margin numbers (§abc123), linkable via URI fragments (#para-abc123).

## Terminology

- **Paragraph:** A logical unit of text; typically separated by blank lines in Markdown, or a semantic block (heading, list item, table row, code block)
- **Purple ID:** The stable identifier for a paragraph; 6–8 base58-encoded characters (e.g., `abc123`)
- **Purple Number:** The rendered form of a purple ID, typically displayed in the right margin (e.g., §abc123)
- **Purple Fragment:** The URI fragment format (#para-abc123)

## Paragraph Definition

### Markdown Documents

A **paragraph** is a maximal block of text separated by one or more blank lines.

```markdown
This is the first paragraph.

This is the second paragraph.

# This is a heading (also a paragraph)

- This is a list item (paragraph)
- Another item

This is a final paragraph.
```

**Result:** 6 paragraphs.

**Special cases:**
- Code blocks (indented or fenced) are single paragraphs, even if they contain blank lines
- Lists: each item is a separate paragraph; the list container itself is not addressed
- Block quotes: treated as single paragraphs (may be refined in future versions)
- HTML blocks: implementation-defined; may be treated as single or multiple paragraphs

### Plain Text Documents

A **paragraph** is a sequence of lines separated by blank lines (similar to Markdown).

### PDF / Image Documents

Support for PDFs and images is **out of scope** for v0.1. Future versions MAY define paragraph extraction via OCR or PDF structure.

## Purple ID Generation

### Algorithm

A purple ID is generated as follows:

1. **Extract paragraph content** as UTF-8 text (stripping leading/trailing whitespace)
2. **Compute content hash:** `hash = SHA-256(paragraph_text)`
3. **Encode:** `base58(hash)` → first 6 characters
4. **Collision detection:** If a paragraph with the same first-6 ID already exists in the document, extend to 8 characters
5. **Result:** A 6–8 character base58 string (e.g., `abc123`, `xyz789abc`)

### Example

Paragraph text: `"The quick brown fox jumps over the lazy dog"`

```
SHA-256 = 0xd7a8...
Base58 = "2QNH7...XyZ2M"
Purple ID = "2QNH7"
```

### Base58 Encoding

[Base58](https://en.wikipedia.org/wiki/Base58) is a base-256 encoding scheme that omits confusing characters (0, O, I, l, +, /). This makes IDs human-readable and copy-paste safe:

```
Alphabet: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz
(note: no 0, O, I, l)
```

## Stability Guarantees

### Content-Based Stability

A purple ID is **stable** if the paragraph content changes minimally:
- Small typo fixes: stable
- Rephrasing (up to ~50% word change): stable
- Reordering paragraphs: ID remains valid (no line number dependency)
- Moving paragraphs across sections: ID remains valid

A purple ID is **invalidated** if:
- Content changes significantly (Levenshtein distance > 50%)
- The paragraph is deleted
- The document is heavily refactored

### Implementation Notes

Implementations SHOULD:
- Use byte-level comparison (UTF-8 equality) when comparing paragraph content
- Allow for minor whitespace normalization (but preserve intentional formatting)
- Provide a "stability check" tool to warn users if a paragraph's ID may be changing due to content edits

## Rendering

### Visual Style

Purple numbers are typically rendered in the **right margin** of the document, superscript or in a small font:

```markdown
This is a paragraph with a purple number.
                                              §abc123

This is another paragraph.
                                              §xyz789
```

### CSS Variables for Theming

Implementations rendering in HTML/CSS SHOULD provide customization:

```css
:root {
  --purple-number-color: #999;
  --purple-number-font-size: 0.75em;
  --purple-number-opacity: 0.6;
  --purple-number-hover-opacity: 1;
  --purple-number-font-family: "Monaco", monospace;
}

.purple-number {
  color: var(--purple-number-color);
  font-size: var(--purple-number-font-size);
  opacity: var(--purple-number-opacity);
  cursor: pointer;
  user-select: none;
}

.purple-number:hover {
  opacity: var(--purple-number-hover-opacity);
}
```

### Visibility Control

- **Show:** By default, purple numbers SHOULD be visible to encourage linking
- **Hide:** Users MAY toggle visibility via settings; hidden numbers should still be clickable/linkable
- **Keyboard navigation:** Tab through paragraphs; Ctrl+C to copy the purple number URI

## URI Fragment Format

A purple-numbered paragraph is referenced via:

```
#para-<purple-id>
```

Examples:
```
hook://file/L3Zhci9sb2cvdGV4dC50eHQ#para-abc123
hook://bookmark/uuid#para-xyz789abc
```

To link to a paragraph:
1. Render the purple number in the document
2. When clicked, the application generates a hook:// URI with the `#para-<id>` fragment
3. The URI is copied to clipboard or shared
4. When opened, the application resolves the file and highlights the paragraph

## Application Responsibilities

### Editor / Note-taking Apps (Obsidian, etc.)

- Compute purple IDs for all paragraphs in the current document
- Render purple numbers in the margin (or sidebar)
- On click, generate and copy the hook:// URI (e.g., to clipboard)
- On URI open, locate the paragraph by ID and highlight it
- On paragraph edit, check if the ID remains stable; warn if content change is significant

### CLI Tools

```bash
hk purple <file>          # Scan and annotate a file with purple numbers
hk purple <file> --json   # Output purple numbers as JSON
```

### Browser Extensions / Web Viewers

- Parse purple numbers from HTML comments or data attributes
- Render them as clickable margin numbers
- Allow bookmarking/linking to paragraphs by ID

## Collision Handling

### Same-Document Collisions

If two paragraphs in the same document have the same first-6 base58 ID:
1. Extend the first paragraph's ID to 8 characters
2. Extend the second paragraph's ID to 8 characters
3. If still colliding (probability ~1 in 58^8 ≈ negligible), document as a known limitation and recommend manual ID overrides in future versions

### Cross-Document Collisions

Cross-document collisions are ignored; each document has its own purple-number namespace.

## Stability Over Time

### Scenario 1: User edits a paragraph

**Before:**
```markdown
The quick brown fox jumps over the lazy dog. (§abc123)
```

**After (typo fix):**
```markdown
The quick brown fox jumps over the lazy dog. (§abc123 — STABLE)
```

**After (major rewrite):**
```markdown
A swift tawny vulpine creature leaps over a canine. (§xyz789 — INVALIDATED)
```

### Scenario 2: User adds a paragraph above

**Before:**
```markdown
First paragraph.              (§aaa111)
Second paragraph.             (§bbb222)
```

**After:**
```markdown
Newly inserted paragraph.     (§ccc333)
First paragraph.              (§aaa111 — STABLE)
Second paragraph.             (§bbb222 — STABLE)
```

Reorganization does NOT change IDs because they are content-based, not position-based.

## Accessibility

### Alt Text for Purple Numbers

When rendering purple numbers in HTML:
```html
<span class="purple-number" aria-label="Paragraph abc123" role="button" tabindex="0">
  §abc123
</span>
```

### Keyboard Navigation

- Tab to purple numbers
- Enter or Space to copy the hook:// URI
- Screen readers announce paragraph IDs

## Performance Considerations

For large documents (10K+ paragraphs):
- Pre-compute and cache purple IDs at document load time
- Incremental updates on edits (only recompute affected paragraphs)
- Use efficient base58 encoder (lookup table, not recursive)

## Future Extensions

Possible future versions might add:
- **Range fragments:** `#para-abc123..xyz789` for multi-paragraph links
- **Semantic paragraphs:** Refined definitions for tables, code, math blocks
- **Structured content:** Purple IDs for table cells, list items within a list
- **Content hashing:** Use Blake3 instead of SHA-256 for faster hashing

## Changelog

### v0.1 (2025-06-06)

- Initial release
- SHA-256 based content hashing
- Base58 encoding
- 6–8 character IDs with collision detection
- Markdown paragraph definition
- Stability guarantees for minor edits
- CSS theming
- URI fragment format

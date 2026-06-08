/**
 * Purple number generation — TypeScript port of the Rust implementation.
 *
 * Algorithm (must match crates/hitchmark-core/src/purple.rs exactly):
 *   SHA-256(paragraph_text) → base58_encode(32 bytes) → first 6 chars
 *
 * Uses @noble/hashes for SHA-256 (sync, no WebCrypto needed).
 */

import { sha256 } from "@noble/hashes/sha256";

/** Bitcoin/bs58 alphabet — same as Rust's bs58 crate default */
const BASE58_ALPHABET =
  "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/**
 * Encode a byte array as base58 (Bitcoin alphabet).
 * Matches the output of Rust's `bs58::encode(&bytes).into_string()`.
 */
function base58Encode(bytes: Uint8Array): string {
  // Count leading zero bytes (each maps to '1')
  let leadingZeros = 0;
  for (const byte of bytes) {
    if (byte === 0) leadingZeros++;
    else break;
  }

  // Encode remaining bytes using BigInt arithmetic
  let num = BigInt(0);
  for (const byte of bytes) {
    num = num * BigInt(256) + BigInt(byte);
  }

  let result = "";
  while (num > BigInt(0)) {
    const remainder = Number(num % BigInt(58));
    num = num / BigInt(58);
    result = BASE58_ALPHABET[remainder] + result;
  }

  return "1".repeat(leadingZeros) + result;
}

/**
 * Generate a purple ID for paragraph text.
 *
 * Produces the same 6-char ID as:
 *   `hk purple <file>` or `PurpleNumberGenerator::generate()` in Rust.
 */
export function generatePurpleId(text: string): string {
  const bytes = new TextEncoder().encode(text);
  const hash = sha256(bytes);
  const encoded = base58Encode(hash);
  return encoded.slice(0, 6);
}

/**
 * Generate a purple ID for a paragraph, extending to 8 chars if there's
 * a collision with another ID already seen in this document.
 *
 * Call with a `seen` Set that persists across the whole document.
 */
export function generatePurpleIdWithCollision(
  text: string,
  seen: Set<string>
): string {
  const bytes = new TextEncoder().encode(text);
  const hash = sha256(bytes);
  const encoded = base58Encode(hash);

  const short = encoded.slice(0, 6);
  if (seen.has(short)) {
    // Collision: extend to 8 chars (same as Rust behaviour)
    return encoded.slice(0, 8);
  }
  seen.add(short);
  return short;
}

/**
 * Split markdown text into paragraphs.
 * Matches `split_paragraphs()` in Rust: split on blank lines, trim, drop empty.
 */
export function splitParagraphs(text: string): string[] {
  return text
    .split(/\n\n+/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

/**
 * Build a map of paragraph text → purple ID for an entire document.
 * Returns an array preserving document order.
 */
export function computeDocumentPurpleIds(
  text: string
): Array<{ text: string; id: string }> {
  const paragraphs = splitParagraphs(text);
  const seen = new Set<string>();
  return paragraphs.map((p) => ({
    text: p,
    id: generatePurpleIdWithCollision(p, seen),
  }));
}

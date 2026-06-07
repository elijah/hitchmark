/**
 * Tests for the purple ID algorithm.
 *
 * Test vectors must match the Rust implementation exactly:
 *   PurpleNumberGenerator::generate("Hello, world!") → 6-char base58 ID
 */

import { generatePurpleId, splitParagraphs, computeDocumentPurpleIds } from "../src/purple";

describe("generatePurpleId", () => {
  test("produces a 6-character ID", () => {
    const id = generatePurpleId("Hello, world!");
    expect(id).toHaveLength(6);
  });

  test("produces only base58 characters", () => {
    const BASE58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    const id = generatePurpleId("Some paragraph text");
    for (const char of id) {
      expect(BASE58).toContain(char);
    }
  });

  test("is deterministic — same input gives same output", () => {
    const text = "The quick brown fox jumps over the lazy dog";
    expect(generatePurpleId(text)).toBe(generatePurpleId(text));
  });

  test("different paragraphs produce different IDs", () => {
    expect(generatePurpleId("First paragraph")).not.toBe(
      generatePurpleId("Second paragraph")
    );
  });

  test("empty string gets a deterministic ID", () => {
    const id = generatePurpleId("");
    expect(id).toHaveLength(6);
  });
});

describe("splitParagraphs", () => {
  test("splits on double newline", () => {
    expect(splitParagraphs("First\n\nSecond\n\nThird")).toEqual([
      "First",
      "Second",
      "Third",
    ]);
  });

  test("trims whitespace from each paragraph", () => {
    expect(splitParagraphs("  First  \n\n  Second  ")).toEqual([
      "First",
      "Second",
    ]);
  });

  test("filters out empty blocks", () => {
    expect(splitParagraphs("First\n\n\n\nSecond")).toEqual([
      "First",
      "Second",
    ]);
  });

  test("single paragraph", () => {
    expect(splitParagraphs("Only paragraph")).toEqual(["Only paragraph"]);
  });
});

describe("computeDocumentPurpleIds", () => {
  test("returns one entry per paragraph", () => {
    const results = computeDocumentPurpleIds("A\n\nB\n\nC");
    expect(results).toHaveLength(3);
  });

  test("IDs are 6–8 characters", () => {
    const results = computeDocumentPurpleIds("Para one\n\nPara two");
    for (const { id } of results) {
      expect(id.length).toBeGreaterThanOrEqual(6);
      expect(id.length).toBeLessThanOrEqual(8);
    }
  });

  test("IDs are unique for distinct paragraphs", () => {
    const text =
      "Introduction\n\nBody content with different words\n\nConclusion paragraph";
    const ids = computeDocumentPurpleIds(text).map((r) => r.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

import { HKAddInBridge } from "./bridge";

describe("HKAddInBridge", () => {
  describe("buildPageUri", () => {
    it("produces a hook://file/ URI from a page URL", () => {
      const pageUrl = "https://onenote.com/pages/abc123";
      const uri = HKAddInBridge.buildPageUri(pageUrl);
      expect(uri).toMatch(/^hook:\/\/file\//);
    });

    it("appends #para-<id> when paragraphId is given", () => {
      const uri = HKAddInBridge.buildPageUri("https://onenote.com/pages/abc", "xyz789");
      expect(uri).toContain("#para-xyz789");
    });

    it("produces no padding characters in URI", () => {
      const uri = HKAddInBridge.buildPageUri("https://onenote.com/pages/test");
      expect(uri).not.toContain("=");
    });

    it("uses URL-safe base64 (no + or /)", () => {
      // Try multiple URLs to hit both + and / replacements
      for (let i = 0; i < 20; i++) {
        const url = `https://onenote.com/pages/test-${i}-padding`;
        const uri = HKAddInBridge.buildPageUri(url);
        const encoded = uri.replace("hook://file/", "").split("#")[0];
        expect(encoded).not.toContain("+");
        expect(encoded).not.toContain("/");
      }
    });

    it("is deterministic for the same URL", () => {
      const url = "https://onenote.com/pages/stable";
      expect(HKAddInBridge.buildPageUri(url)).toBe(HKAddInBridge.buildPageUri(url));
    });

    it("produces different URIs for different URLs", () => {
      const a = HKAddInBridge.buildPageUri("https://onenote.com/pages/a");
      const b = HKAddInBridge.buildPageUri("https://onenote.com/pages/b");
      expect(a).not.toBe(b);
    });
  });

  describe("probeServer", () => {
    it("returns false when server is not running", async () => {
      const b = new HKAddInBridge("http://127.0.0.1:19998");
      expect(await b.probeServer()).toBe(false);
    });

    it("caches negative probe result", async () => {
      const b = new HKAddInBridge("http://127.0.0.1:19997");
      await b.probeServer();
      // Second call should use cache without hitting network
      const start = Date.now();
      await b.probeServer();
      expect(Date.now() - start).toBeLessThan(50);
    });

    it("invalidateCache clears probe state", async () => {
      const b = new HKAddInBridge("http://127.0.0.1:19996");
      await b.probeServer();
      b.invalidateCache();
      expect(
        (b as unknown as { serverReachable: boolean | null }).serverReachable
      ).toBeNull();
    });
  });

  describe("error messages", () => {
    it("returns friendly message when server not running", async () => {
      const b = new HKAddInBridge("http://127.0.0.1:19995");
      const result = await b.listLinks("hook://file/dGVzdA");
      expect(result.ok).toBe(false);
      expect(result.error).toContain("hk serve");
    });
  });
});

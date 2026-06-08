import { HKSafariBridge } from "../src/bridge";

// Minimal fetch mock
const mockFetch = jest.fn();
global.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

describe("HKSafariBridge.buildWebUri", () => {
  it("produces hook://file/<base64url> for a web URL", () => {
    const url = "https://example.com/page";
    const uri = HKSafariBridge.buildWebUri(url);
    expect(uri).toMatch(/^hook:\/\/file\//);
  });

  it("uses URL-safe base64 (no +, /, or padding)", () => {
    const url = "https://example.com/path?q=hello+world&x=1/2";
    const uri = HKSafariBridge.buildWebUri(url);
    const encoded = uri.slice("hook://file/".length);
    expect(encoded).not.toMatch(/[+/=]/);
  });

  it("is deterministic", () => {
    const url = "https://example.com/test";
    expect(HKSafariBridge.buildWebUri(url)).toBe(HKSafariBridge.buildWebUri(url));
  });

  it("produces different URIs for different URLs", () => {
    expect(HKSafariBridge.buildWebUri("https://a.com")).not.toBe(
      HKSafariBridge.buildWebUri("https://b.com"),
    );
  });
});

describe("probeServer", () => {
  it("returns false when server is not running", async () => {
    mockFetch.mockRejectedValueOnce(new Error("Connection refused"));
    const bridge = new HKSafariBridge();
    expect(await bridge.probeServer()).toBe(false);
  });

  it("returns true when server responds ok", async () => {
    mockFetch.mockResolvedValueOnce({ ok: true } as Response);
    const bridge = new HKSafariBridge();
    expect(await bridge.probeServer()).toBe(true);
  });

  it("caches probe result", async () => {
    mockFetch.mockRejectedValueOnce(new Error("refused"));
    const bridge = new HKSafariBridge();
    await bridge.probeServer();
    await bridge.probeServer(); // second call — no fetch
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it("invalidateCache clears probe state", async () => {
    mockFetch
      .mockRejectedValueOnce(new Error("refused"))
      .mockResolvedValueOnce({ ok: true } as Response);
    const bridge = new HKSafariBridge();
    expect(await bridge.probeServer()).toBe(false);
    bridge.invalidateCache();
    expect(await bridge.probeServer()).toBe(true);
  });
});

describe("createLink", () => {
  it("posts to /links and returns ok", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => null,
    } as unknown as Response);
    const bridge = new HKSafariBridge();
    const res = await bridge.createLink("hook://file/a", "hook://file/b", "test");
    expect(res.ok).toBe(true);
    const [url, opts] = mockFetch.mock.calls[0] as [string, RequestInit];
    expect(url).toContain("/links");
    expect(opts.method).toBe("POST");
  });
});

describe("listLinks", () => {
  it("GETs /links?uri=... and returns array", async () => {
    const links = [{ uri_a: "hook://file/a", uri_b: "hook://file/b" }];
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => links,
    } as unknown as Response);
    const bridge = new HKSafariBridge();
    const res = await bridge.listLinks("hook://file/a");
    expect(res.ok).toBe(true);
    if (res.ok) expect(res.data).toEqual(links);
  });
});

describe("serverNotRunningMessage", () => {
  it("mentions hk serve", () => {
    const bridge = new HKSafariBridge();
    expect(bridge.serverNotRunningMessage()).toContain("hk serve");
  });
});

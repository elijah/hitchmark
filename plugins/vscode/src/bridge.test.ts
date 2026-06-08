import { HKBridge } from "./bridge";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

describe("HKBridge (VS Code)", () => {
  let bridge: HKBridge;

  beforeEach(() => {
    bridge = new HKBridge("", "");
  });

  describe("resolvePath", () => {
    it("returns null when no hk binary found and no cliPath set", () => {
      const b = new HKBridge("/nonexistent/hk", "");
      // nonexistent path → falls through to search paths
      expect(b.resolvePath()).toBeNull();
    });

    it("returns cliPath when it points to an existing file", () => {
      const tmp = path.join(os.tmpdir(), "hk-mock");
      fs.writeFileSync(tmp, "#!/bin/sh\necho mock", { mode: 0o755 });
      const b = new HKBridge(tmp, "");
      expect(b.resolvePath()).toBe(tmp);
      fs.unlinkSync(tmp);
    });

    it("caches the resolved path", () => {
      const tmp = path.join(os.tmpdir(), "hk-mock2");
      fs.writeFileSync(tmp, "#!/bin/sh", { mode: 0o755 });
      const b = new HKBridge(tmp, "");
      const first = b.resolvePath();
      const second = b.resolvePath();
      expect(first).toBe(second);
      fs.unlinkSync(tmp);
    });
  });

  describe("invalidateCache", () => {
    it("clears cached resolution", () => {
      const b = new HKBridge("", "http://127.0.0.1:2701");
      (b as unknown as { resolvedPath: string | null }).resolvedPath = "/cached/path";
      (b as unknown as { serverReachable: boolean | null }).serverReachable = true;
      b.invalidateCache();
      expect((b as unknown as { resolvedPath: string | null }).resolvedPath).toBeNull();
      expect((b as unknown as { serverReachable: boolean | null }).serverReachable).toBeNull();
    });
  });

  describe("probeServer", () => {
    it("returns false when serverUrl is empty", async () => {
      const b = new HKBridge("", "");
      expect(await b.probeServer()).toBe(false);
    });

    it("returns false when server is not running", async () => {
      const b = new HKBridge("", "http://127.0.0.1:19999");
      expect(await b.probeServer()).toBe(false);
    });
  });

  describe("run fallback", () => {
    it("returns error when hk binary is not found", async () => {
      const b = new HKBridge("/no/such/binary", "");
      const result = await b.fileToUri("/tmp/test.md");
      expect(result.ok).toBe(false);
      expect(result.error).toContain("not found");
    });
  });
});

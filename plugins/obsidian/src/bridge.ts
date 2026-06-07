/**
 * Bridge from the Obsidian plugin to the `hk` CLI.
 *
 * Transport priority:
 *   1. HTTP (`hk serve`) — if `serverUrl` is set and the server is reachable
 *   2. Subprocess — spawn `hk` directly (always works, higher latency)
 *
 * Set `serverUrl` in plugin settings to "http://127.0.0.1:2701" once
 * `hk serve` is running for best performance.
 */

import { exec } from "child_process";
import { promisify } from "util";
import type { HKResponse, LinkRecord } from "./types";

const execAsync = promisify(exec);

/** Candidate paths for locating the `hk` binary */
const HK_SEARCH_PATHS = [
  "/usr/local/bin/hk",
  `${process.env.HOME ?? "~"}/.cargo/bin/hk`,
  "/opt/homebrew/bin/hk",
  "/usr/bin/hk",
  "/usr/local/opt/hookmarks/bin/hk",
];

export class HKBridge {
  private resolvedPath: string | null = null;
  private serverReachable: boolean | null = null; // null = not yet probed

  constructor(
    private explicitPath = "",
    /** Optional `hk serve` base URL, e.g. "http://127.0.0.1:2701" */
    public serverUrl = ""
  ) {}

  /** Resolve the hk binary path (cached after first call) */
  async resolvePath(): Promise<string | null> {
    if (this.resolvedPath) return this.resolvedPath;

    if (this.explicitPath) {
      this.resolvedPath = this.explicitPath;
      return this.resolvedPath;
    }

    const { existsSync } = await import("fs");
    for (const p of HK_SEARCH_PATHS) {
      if (existsSync(p)) {
        this.resolvedPath = p;
        return p;
      }
    }
    return null;
  }

  /** Invalidate cached paths/probes (call when settings change) */
  invalidateCache(): void {
    this.resolvedPath = null;
    this.serverReachable = null;
  }

  /** Probe whether the `hk serve` HTTP server is reachable. Result is cached. */
  async probeServer(): Promise<boolean> {
    if (!this.serverUrl) return false;
    if (this.serverReachable !== null) return this.serverReachable;
    try {
      const res = await fetch(`${this.serverUrl}/health`, {
        signal: AbortSignal.timeout(1500),
      });
      this.serverReachable = res.ok;
    } catch {
      this.serverReachable = false;
    }
    return this.serverReachable;
  }

  /** Perform an HTTP request to the `hk serve` server */
  private async httpRequest<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<HKResponse<T>> {
    try {
      const res = await fetch(`${this.serverUrl}${path}`, {
        method,
        headers: body ? { "Content-Type": "application/json" } : undefined,
        body: body ? JSON.stringify(body) : undefined,
        signal: AbortSignal.timeout(8_000),
      });
      const json = await res.json();
      if (!res.ok) {
        return { ok: false, error: json.error ?? `HTTP ${res.status}` };
      }
      return { ok: true, value: json as T };
    } catch (err) {
      // Server went away — invalidate so next call retries
      this.serverReachable = null;
      return { ok: false, error: err instanceof Error ? err.message : String(err) };
    }
  }

  /** Run an `hk` subcommand and return stdout */
  private async run(
    subcommand: string,
    args: string[]
  ): Promise<HKResponse<string>> {
    const hkPath = await this.resolvePath();
    if (!hkPath) {
      return {
        ok: false,
        error:
          "hk binary not found. Install with: cargo install hookmarks-cli or brew install hookmarks",
      };
    }

    // Shell-escape each argument
    const escaped = args.map((a) => `'${a.replace(/'/g, "'\\''")}'`).join(" ");
    const cmd = `'${hkPath}' ${subcommand} ${escaped}`;

    try {
      const { stdout, stderr } = await execAsync(cmd, { timeout: 10_000 });
      if (stderr.trim()) {
        return { ok: false, error: stderr.trim() };
      }
      return { ok: true, value: stdout.trim() };
    } catch (err) {
      const message =
        err instanceof Error ? err.message : String(err);
      return { ok: false, error: message };
    }
  }

  /** Convert a file path to a hook:// URI */
  async fileToUri(filePath: string): Promise<HKResponse<string>> {
    if (await this.probeServer()) {
      const res = await this.httpRequest<{ uri: string }>(
        "GET",
        `/uri?path=${encodeURIComponent(filePath)}`
      );
      if (res.ok) return { ok: true, value: res.value.uri };
    }
    return this.run("file", [filePath]);
  }

  /** Resolve and open a hook:// URI (always subprocess — needs OS integration) */
  async openUri(uri: string): Promise<HKResponse<string>> {
    return this.run("open", [uri]);
  }

  /** List all links for a resource URI */
  async listLinks(uri: string): Promise<HKResponse<LinkRecord[]>> {
    if (await this.probeServer()) {
      return this.httpRequest<LinkRecord[]>(
        "GET",
        `/links?uri=${encodeURIComponent(uri)}`
      );
    }

    const result = await this.run("list", [uri, "--json"]);
    if (!result.ok) return result;
    if (!result.value) return { ok: true, value: [] };
    try {
      const records: LinkRecord[] = JSON.parse(result.value);
      return { ok: true, value: records };
    } catch {
      return { ok: false, error: "Failed to parse hk list output as JSON" };
    }
  }

  /** Create a bidirectional link between two URIs */
  async createLink(
    uriA: string,
    uriB: string,
    note?: string
  ): Promise<HKResponse<string>> {
    if (await this.probeServer()) {
      const res = await this.httpRequest<{ ok: boolean }>("POST", "/links", {
        uri_a: uriA,
        uri_b: uriB,
        note,
      });
      if (res.ok) return { ok: true, value: "" };
      // 409 Conflict is not an error for callers that don't care about duplicates
      return res as HKResponse<string>;
    }
    const args = note ? [uriA, uriB, "--note", note] : [uriA, uriB];
    return this.run("link", args);
  }

  /** Generate purple numbers for a file (returns JSON) */
  async purple(filePath: string): Promise<HKResponse<string>> {
    return this.run("purple", [filePath, "--format", "json"]);
  }

  /** Check if hk is reachable (subprocess or server) */
  async ping(): Promise<boolean> {
    if (await this.probeServer()) return true;
    const result = await this.run("--version", []);
    return result.ok;
  }
}

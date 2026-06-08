/**
 * Bridge from the VS Code extension to the `hk` CLI.
 *
 * Mirrors the Obsidian bridge pattern:
 *   1. Try HTTP (hk serve) if configured and reachable
 *   2. Fall back to subprocess
 */

import { execFile } from "child_process";
import { promisify } from "util";
import * as fs from "fs";

const execFileAsync = promisify(execFile);

const HK_SEARCH_PATHS = [
  "/usr/local/bin/hk",
  `${process.env.HOME ?? ""}/.cargo/bin/hk`,
  "/opt/homebrew/bin/hk",
  "/usr/bin/hk",
];

export interface LinkRecord {
  source: string;
  target: string;
  note?: string;
  created_at: string;
}

export interface PurpleRecord {
  id: string;
  text: string;
}

export interface HKResponse<T> {
  ok: boolean;
  value?: T;
  error?: string;
}

export class HKBridge {
  private resolvedPath: string | null = null;
  private serverReachable: boolean | null = null;

  constructor(
    private cliPath = "",
    public serverUrl = "http://127.0.0.1:2701"
  ) {}

  invalidateCache(): void {
    this.resolvedPath = null;
    this.serverReachable = null;
  }

  resolvePath(): string | null {
    if (this.resolvedPath) return this.resolvedPath;
    if (this.cliPath && fs.existsSync(this.cliPath)) {
      this.resolvedPath = this.cliPath;
      return this.resolvedPath;
    }
    for (const p of HK_SEARCH_PATHS) {
      if (fs.existsSync(p)) {
        this.resolvedPath = p;
        return p;
      }
    }
    return null;
  }

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

  private async httpGet<T>(path: string): Promise<HKResponse<T>> {
    try {
      const res = await fetch(`${this.serverUrl}${path}`, {
        signal: AbortSignal.timeout(5000),
      });
      const json = await res.json();
      if (!res.ok) return { ok: false, error: (json as { error: string }).error };
      return { ok: true, value: json as T };
    } catch (e) {
      this.serverReachable = null;
      return { ok: false, error: String(e) };
    }
  }

  private async httpPost<T>(path: string, body: unknown): Promise<HKResponse<T>> {
    try {
      const res = await fetch(`${this.serverUrl}${path}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
        signal: AbortSignal.timeout(5000),
      });
      const json = await res.json();
      if (!res.ok) return { ok: false, error: (json as { error: string }).error };
      return { ok: true, value: json as T };
    } catch (e) {
      this.serverReachable = null;
      return { ok: false, error: String(e) };
    }
  }

  private async run(subcommand: string, args: string[]): Promise<HKResponse<string>> {
    const hkPath = this.resolvePath();
    if (!hkPath) {
      return {
        ok: false,
        error: "hk binary not found. Install with: cargo install hookmarks-cli",
      };
    }
    try {
      const { stdout, stderr } = await execFileAsync(hkPath, [subcommand, ...args], {
        timeout: 10_000,
      });
      if (stderr.trim()) return { ok: false, error: stderr.trim() };
      return { ok: true, value: stdout.trim() };
    } catch (e) {
      return { ok: false, error: String(e) };
    }
  }

  async fileToUri(filePath: string): Promise<HKResponse<string>> {
    if (await this.probeServer()) {
      const r = await this.httpGet<{ uri: string }>(
        `/uri?path=${encodeURIComponent(filePath)}`
      );
      if (r.ok && r.value) return { ok: true, value: r.value.uri };
    }
    return this.run("file", [filePath]);
  }

  async listLinks(uri: string): Promise<HKResponse<LinkRecord[]>> {
    if (await this.probeServer()) {
      return this.httpGet<LinkRecord[]>(`/links?uri=${encodeURIComponent(uri)}`);
    }
    const r = await this.run("list", [uri, "--json"]);
    if (!r.ok) return { ok: false, error: r.error } as HKResponse<LinkRecord[]>;
    try {
      return { ok: true, value: JSON.parse(r.value ?? "[]") };
    } catch {
      return { ok: false, error: "Failed to parse JSON" };
    }
  }

  async createLink(uriA: string, uriB: string, note?: string): Promise<HKResponse<void>> {
    if (await this.probeServer()) {
      const r = await this.httpPost<{ ok: boolean }>("/links", {
        uri_a: uriA, uri_b: uriB, note,
      });
      return r.ok ? { ok: true } : { ok: false, error: r.error };
    }
    const args = note ? [uriA, uriB, "--note", note] : [uriA, uriB];
    const r = await this.run("link", args);
    return r.ok ? { ok: true } : { ok: false, error: r.error };
  }

  async getPurpleNumbers(filePath: string): Promise<HKResponse<PurpleRecord[]>> {
    if (await this.probeServer()) {
      return this.httpGet<PurpleRecord[]>(
        `/purple?path=${encodeURIComponent(filePath)}`
      );
    }
    const r = await this.run("purple", [filePath, "--format", "json"]);
    if (!r.ok) return { ok: false, error: r.error } as HKResponse<PurpleRecord[]>;
    try {
      return { ok: true, value: JSON.parse(r.value ?? "[]") };
    } catch {
      return { ok: false, error: "Failed to parse JSON" };
    }
  }

  async openUri(uri: string): Promise<HKResponse<void>> {
    const r = await this.run("open", [uri]);
    return r.ok ? { ok: true } : { ok: false, error: r.error };
  }
}

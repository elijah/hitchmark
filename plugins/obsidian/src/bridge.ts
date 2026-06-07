/**
 * Bridge from the Obsidian plugin to the `hk` CLI.
 *
 * Uses Node.js `child_process` (available in Obsidian/Electron) to spawn
 * the `hk` binary for all operations. Falls back to a local HTTP server
 * if `daemonUrl` is configured and reachable (Phase 2).
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

  constructor(private explicitPath = "") {}

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

  /** Invalidate cached path (call when settings change) */
  invalidateCache(): void {
    this.resolvedPath = null;
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
    return this.run("file", [filePath]);
  }

  /** Resolve and open a hook:// URI */
  async openUri(uri: string): Promise<HKResponse<string>> {
    return this.run("open", [uri]);
  }

  /** List all links for a resource URI */
  async listLinks(uri: string): Promise<HKResponse<LinkRecord[]>> {
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
    const args = note ? [uriA, uriB, "--note", note] : [uriA, uriB];
    return this.run("link", args);
  }

  /** Generate purple numbers for a file (returns JSON) */
  async purple(filePath: string): Promise<HKResponse<string>> {
    return this.run("purple", [filePath, "--format", "json"]);
  }

  /** Check if hk is reachable */
  async ping(): Promise<boolean> {
    const result = await this.run("--version", []);
    return result.ok;
  }
}

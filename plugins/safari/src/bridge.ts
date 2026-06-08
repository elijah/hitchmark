/**
 * HitchmarkBridge — Safari Web Extension
 *
 * Communicates with `hk serve` running on localhost:2701.
 * All operations are HTTP-only (no subprocess access from browser extensions).
 */

const DEFAULT_SERVER = "http://127.0.0.1:2701";

export interface HKLink {
  uri_a: string;
  uri_b: string;
  note?: string;
}

export interface HKPurpleNumber {
  id: string;
  text: string;
}

export type HKResponse<T> =
  | { ok: true; data: T }
  | { ok: false; error: string };

export class HKSafariBridge {
  private serverUrl: string;
  private _serverAvailable: boolean | null = null;

  constructor(serverUrl = DEFAULT_SERVER) {
    this.serverUrl = serverUrl.replace(/\/$/, "");
  }

  updateServerUrl(url: string): void {
    this.serverUrl = url.replace(/\/$/, "");
    this._serverAvailable = null;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
  ): Promise<HKResponse<T>> {
    try {
      const res = await fetch(`${this.serverUrl}${path}`, {
        method,
        headers: body ? { "Content-Type": "application/json" } : undefined,
        body: body ? JSON.stringify(body) : undefined,
        signal: AbortSignal.timeout(3000),
      });
      if (!res.ok) {
        const text = await res.text().catch(() => res.statusText);
        return { ok: false, error: text };
      }
      const data = await res.json() as T;
      return { ok: true, data };
    } catch (err) {
      return { ok: false, error: (err as Error).message };
    }
  }

  async probeServer(): Promise<boolean> {
    if (this._serverAvailable !== null) return this._serverAvailable;
    try {
      const res = await fetch(`${this.serverUrl}/health`, {
        signal: AbortSignal.timeout(1500),
      });
      this._serverAvailable = res.ok;
    } catch {
      this._serverAvailable = false;
    }
    return this._serverAvailable;
  }

  invalidateCache(): void {
    this._serverAvailable = null;
  }

  /**
   * Convert a file path (or URL) to a hook:// URI via the server.
   */
  async fileToUri(filePath: string): Promise<HKResponse<string>> {
    return this.request<string>("GET", `/uri?path=${encodeURIComponent(filePath)}`);
  }

  /**
   * Build a hook:// URI for the current browser tab URL.
   * Encodes the URL using URL-safe base64 (no padding).
   */
  static buildWebUri(tabUrl: string): string {
    const b64 = btoa(tabUrl)
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=+$/, "");
    return `hook://file/${b64}`;
  }

  async listLinks(uri: string): Promise<HKResponse<HKLink[]>> {
    return this.request<HKLink[]>(
      "GET",
      `/links?uri=${encodeURIComponent(uri)}`,
    );
  }

  async createLink(
    uriA: string,
    uriB: string,
    note?: string,
  ): Promise<HKResponse<null>> {
    return this.request<null>("POST", "/links", {
      uri_a: uriA,
      uri_b: uriB,
      note: note ?? null,
    });
  }

  async deleteLink(uriA: string, uriB: string): Promise<HKResponse<null>> {
    return this.request<null>("DELETE", "/links", {
      uri_a: uriA,
      uri_b: uriB,
    });
  }

  async getPurpleNumbers(filePath: string): Promise<HKResponse<HKPurpleNumber[]>> {
    return this.request<HKPurpleNumber[]>(
      "GET",
      `/purple?path=${encodeURIComponent(filePath)}`,
    );
  }

  serverNotRunningMessage(): string {
    return (
      `Hitchmark server is not running.\n\n` +
      `Start it with:  hk serve\n\n` +
      `The server must be running on ${this.serverUrl} for the Safari extension to work.`
    );
  }
}

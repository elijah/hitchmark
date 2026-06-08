/**
 * Hitchmark OneNote Add-in — Office JS bridge.
 *
 * OneNote add-ins run in a browser sandbox and CANNOT spawn subprocesses.
 * All operations go through the `hk serve` HTTP API.
 *
 * The add-in generates hook:// URIs for OneNote pages using the page's
 * stable OneNote URL (from the Office JS API) encoded as base64url,
 * matching the hook://file/<base64url> scheme.
 */

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

export class HKAddInBridge {
  private serverReachable: boolean | null = null;

  constructor(public serverUrl = "http://127.0.0.1:2701") {}

  invalidateCache(): void {
    this.serverReachable = null;
  }

  async probeServer(): Promise<boolean> {
    if (this.serverReachable !== null) return this.serverReachable;
    try {
      const res = await fetch(`${this.serverUrl}/health`, {
        signal: AbortSignal.timeout(2000),
      });
      this.serverReachable = res.ok;
    } catch {
      this.serverReachable = false;
    }
    return this.serverReachable;
  }

  // ── HTTP helpers ────────────────────────────────────────────────────────────

  private async httpGet<T>(path: string): Promise<HKResponse<T>> {
    try {
      const res = await fetch(`${this.serverUrl}${path}`, {
        signal: AbortSignal.timeout(8000),
      });
      const json = await res.json();
      if (!res.ok) return { ok: false, error: (json as { error: string }).error };
      return { ok: true, value: json as T };
    } catch (e) {
      this.serverReachable = null;
      return { ok: false, error: `Server unreachable: ${e}` };
    }
  }

  private async httpPost<T>(path: string, body: unknown): Promise<HKResponse<T>> {
    try {
      const res = await fetch(`${this.serverUrl}${path}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
        signal: AbortSignal.timeout(8000),
      });
      const json = await res.json();
      if (!res.ok) return { ok: false, error: (json as { error: string }).error };
      return { ok: true, value: json as T };
    } catch (e) {
      this.serverReachable = null;
      return { ok: false, error: `Server unreachable: ${e}` };
    }
  }

  private async httpDelete<T>(path: string, body: unknown): Promise<HKResponse<T>> {
    try {
      const res = await fetch(`${this.serverUrl}${path}`, {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
        signal: AbortSignal.timeout(8000),
      });
      const json = await res.json();
      if (!res.ok) return { ok: false, error: (json as { error: string }).error };
      return { ok: true, value: json as T };
    } catch (e) {
      this.serverReachable = null;
      return { ok: false, error: `Server unreachable: ${e}` };
    }
  }

  // ── OneNote page URI ────────────────────────────────────────────────────────

  /**
   * Build a hook:// URI for a OneNote page.
   *
   * Uses the OneNote page's web URL as the stable identifier,
   * base64url-encoded: hook://file/<base64url(pageUrl)>
   *
   * The server's /uri endpoint is for local file paths; for OneNote we
   * build the URI client-side since the page URL is already stable.
   */
  static buildPageUri(pageWebUrl: string, paragraphId?: string): string {
    const encoded = btoa(pageWebUrl)
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=+$/, "");
    const base = `hook://file/${encoded}`;
    return paragraphId ? `${base}#para-${paragraphId}` : base;
  }

  // ── link store operations ───────────────────────────────────────────────────

  async listLinks(uri: string): Promise<HKResponse<LinkRecord[]>> {
    if (!(await this.probeServer())) {
      return { ok: false, error: serverNotRunningMessage() };
    }
    return this.httpGet<LinkRecord[]>(`/links?uri=${encodeURIComponent(uri)}`);
  }

  async createLink(uriA: string, uriB: string, note?: string): Promise<HKResponse<void>> {
    if (!(await this.probeServer())) {
      return { ok: false, error: serverNotRunningMessage() };
    }
    const r = await this.httpPost<{ ok: boolean }>("/links", {
      uri_a: uriA, uri_b: uriB, note,
    });
    return r.ok ? { ok: true } : { ok: false, error: r.error };
  }

  async deleteLink(uriA: string, uriB: string): Promise<HKResponse<void>> {
    if (!(await this.probeServer())) {
      return { ok: false, error: serverNotRunningMessage() };
    }
    const r = await this.httpDelete<{ ok: boolean }>("/links", {
      uri_a: uriA, uri_b: uriB,
    });
    return r.ok ? { ok: true } : { ok: false, error: r.error };
  }
}

function serverNotRunningMessage(): string {
  return (
    "hk serve is not running. Start it from your terminal: hk serve\n" +
    "The Hitchmark OneNote add-in requires the local HTTP server."
  );
}

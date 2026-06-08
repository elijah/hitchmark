"use strict";
(() => {
  // src/bridge.ts
  var DEFAULT_SERVER = "http://127.0.0.1:2701";
  var HKSafariBridge = class {
    constructor(serverUrl = DEFAULT_SERVER) {
      this._serverAvailable = null;
      this.serverUrl = serverUrl.replace(/\/$/, "");
    }
    updateServerUrl(url) {
      this.serverUrl = url.replace(/\/$/, "");
      this._serverAvailable = null;
    }
    async request(method, path, body) {
      try {
        const res = await fetch(`${this.serverUrl}${path}`, {
          method,
          headers: body ? { "Content-Type": "application/json" } : void 0,
          body: body ? JSON.stringify(body) : void 0,
          signal: AbortSignal.timeout(3e3)
        });
        if (!res.ok) {
          const text = await res.text().catch(() => res.statusText);
          return { ok: false, error: text };
        }
        const data = await res.json();
        return { ok: true, data };
      } catch (err) {
        return { ok: false, error: err.message };
      }
    }
    async probeServer() {
      if (this._serverAvailable !== null) return this._serverAvailable;
      try {
        const res = await fetch(`${this.serverUrl}/health`, {
          signal: AbortSignal.timeout(1500)
        });
        this._serverAvailable = res.ok;
      } catch {
        this._serverAvailable = false;
      }
      return this._serverAvailable;
    }
    invalidateCache() {
      this._serverAvailable = null;
    }
    /**
     * Convert a file path (or URL) to a hook:// URI via the server.
     */
    async fileToUri(filePath) {
      return this.request("GET", `/uri?path=${encodeURIComponent(filePath)}`);
    }
    /**
     * Build a hook:// URI for the current browser tab URL.
     * Encodes the URL using URL-safe base64 (no padding).
     */
    static buildWebUri(tabUrl) {
      const b64 = btoa(tabUrl).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
      return `hook://file/${b64}`;
    }
    async listLinks(uri) {
      return this.request(
        "GET",
        `/links?uri=${encodeURIComponent(uri)}`
      );
    }
    async createLink(uriA, uriB, note) {
      return this.request("POST", "/links", {
        uri_a: uriA,
        uri_b: uriB,
        note: note ?? null
      });
    }
    async deleteLink(uriA, uriB) {
      return this.request("DELETE", "/links", {
        uri_a: uriA,
        uri_b: uriB
      });
    }
    async getPurpleNumbers(filePath) {
      return this.request(
        "GET",
        `/purple?path=${encodeURIComponent(filePath)}`
      );
    }
    serverNotRunningMessage() {
      return `Hookmarks server is not running.

Start it with:  hk serve

The server must be running on ${this.serverUrl} for the Safari extension to work.`;
    }
  };

  // src/background.ts
  var bridge = new HKSafariBridge();
  chrome.storage.local.get(["serverUrl"], (result) => {
    if (result.serverUrl) {
      bridge.updateServerUrl(result.serverUrl);
    }
  });
  chrome.runtime.onInstalled.addListener(() => {
    chrome.contextMenus?.create({
      id: "hookmarks-copy-uri",
      title: "Copy hook:// link",
      contexts: ["page", "link"]
    });
    chrome.contextMenus?.create({
      id: "hookmarks-link-to-page",
      title: "Link to current page\u2026",
      contexts: ["page"]
    });
  });
  chrome.contextMenus?.onClicked.addListener((info, tab) => {
    if (!tab?.url) return;
    const targetUrl = info.linkUrl ?? tab.url;
    if (info.menuItemId === "hookmarks-copy-uri") {
      const uri = HKSafariBridge.buildWebUri(targetUrl);
      chrome.scripting.executeScript({
        target: { tabId: tab.id },
        func: (text) => navigator.clipboard.writeText(text),
        args: [uri]
      });
    }
  });
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    handleMessage(message).then(sendResponse);
    return true;
  });
  async function handleMessage(msg) {
    switch (msg.type) {
      case "probe":
        bridge.invalidateCache();
        return { available: await bridge.probeServer() };
      case "buildUri": {
        const url = msg.payload?.url;
        return { uri: HKSafariBridge.buildWebUri(url) };
      }
      case "listLinks": {
        const uri = msg.payload?.uri;
        return bridge.listLinks(uri);
      }
      case "createLink": {
        const { uriA, uriB, note } = msg.payload;
        return bridge.createLink(uriA, uriB, note);
      }
      case "deleteLink": {
        const { uriA, uriB } = msg.payload;
        return bridge.deleteLink(uriA, uriB);
      }
      case "updateSettings": {
        const serverUrl = msg.payload?.serverUrl;
        bridge.updateServerUrl(serverUrl);
        chrome.storage.local.set({ serverUrl });
        return { ok: true };
      }
      default:
        return { ok: false, error: `Unknown message type: ${msg.type}` };
    }
  }
})();
//# sourceMappingURL=background.js.map

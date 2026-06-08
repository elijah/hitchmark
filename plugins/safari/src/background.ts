/**
 * Background service worker for the Hitchmark Safari extension.
 *
 * Handles context menu clicks and messages from the popup.
 * Safari MV3 background runs as a service worker — no persistent state.
 */

import { HKSafariBridge } from "./bridge.js";

let bridge = new HKSafariBridge();

// Restore server URL from storage on startup
chrome.storage.local.get(["serverUrl"], (result) => {
  if (result.serverUrl) {
    bridge.updateServerUrl(result.serverUrl as string);
  }
});

// Context menu — "Copy hook:// link"
chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus?.create({
    id: "hitchmark-copy-uri",
    title: "Copy hook:// link",
    contexts: ["page", "link"],
  });
  chrome.contextMenus?.create({
    id: "hitchmark-link-to-page",
    title: "Link to current page…",
    contexts: ["page"],
  });
});

chrome.contextMenus?.onClicked.addListener((info, tab) => {
  if (!tab?.url) return;
  const targetUrl = info.linkUrl ?? tab.url;

  if (info.menuItemId === "hitchmark-copy-uri") {
    const uri = HKSafariBridge.buildWebUri(targetUrl);
    // Write to clipboard via content script injection
    chrome.scripting.executeScript({
      target: { tabId: tab.id! },
      func: (text: string) => navigator.clipboard.writeText(text),
      args: [uri],
    });
  }
});

// Message bus — popup and content script communicate via messages
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  handleMessage(message as BackgroundMessage).then(sendResponse);
  return true; // keep channel open for async response
});

interface BackgroundMessage {
  type: "probe" | "listLinks" | "createLink" | "deleteLink" | "buildUri" | "updateSettings";
  payload?: Record<string, unknown>;
}

async function handleMessage(msg: BackgroundMessage): Promise<unknown> {
  switch (msg.type) {
    case "probe":
      bridge.invalidateCache();
      return { available: await bridge.probeServer() };

    case "buildUri": {
      const url = msg.payload?.url as string;
      return { uri: HKSafariBridge.buildWebUri(url) };
    }

    case "listLinks": {
      const uri = msg.payload?.uri as string;
      return bridge.listLinks(uri);
    }

    case "createLink": {
      const { uriA, uriB, note } = msg.payload as {
        uriA: string;
        uriB: string;
        note?: string;
      };
      return bridge.createLink(uriA, uriB, note);
    }

    case "deleteLink": {
      const { uriA, uriB } = msg.payload as { uriA: string; uriB: string };
      return bridge.deleteLink(uriA, uriB);
    }

    case "updateSettings": {
      const serverUrl = msg.payload?.serverUrl as string;
      bridge.updateServerUrl(serverUrl);
      chrome.storage.local.set({ serverUrl });
      return { ok: true };
    }

    default:
      return { ok: false, error: `Unknown message type: ${(msg as BackgroundMessage).type}` };
  }
}

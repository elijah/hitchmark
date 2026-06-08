/**
 * Popup script for the Hitchmark Safari extension.
 * Communicates with the background service worker via chrome.runtime.sendMessage.
 */

type Message = { type: string; payload?: Record<string, unknown> };

function send<T>(msg: Message): Promise<T> {
  return new Promise((resolve) => chrome.runtime.sendMessage(msg, resolve));
}

const elUri = document.getElementById("current-uri")!;
const elBadge = document.getElementById("server-badge")!;
const elLinksList = document.getElementById("links-list")!;
const elLinkTarget = document.getElementById("link-target") as HTMLInputElement;
const elLinkNote = document.getElementById("link-note") as HTMLInputElement;
const elServerUrl = document.getElementById("server-url") as HTMLInputElement;
const elStatus = document.getElementById("status")!;

let currentUri = "";

function setStatus(msg: string, ms = 3000): void {
  elStatus.textContent = msg;
  if (ms > 0) setTimeout(() => { elStatus.textContent = ""; }, ms);
}

async function init(): Promise<void> {
  // Load saved server URL
  chrome.storage.local.get(["serverUrl"], (r) => {
    elServerUrl.value = (r.serverUrl as string) ?? "http://127.0.0.1:2701";
  });

  // Probe server
  const { available } = await send<{ available: boolean }>({ type: "probe" });
  elBadge.textContent = available ? "online" : "offline";
  elBadge.className = `server-badge ${available ? "online" : "offline"}`;

  // Get current tab URL
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.url) return;

  const { uri } = await send<{ uri: string }>({
    type: "buildUri",
    payload: { url: tab.url },
  });
  currentUri = uri;
  elUri.textContent = uri;
  elUri.title = uri;

  if (!available) {
    elLinksList.innerHTML = '<li class="empty">Start hk serve to load links.</li>';
    return;
  }

  await loadLinks();
}

async function loadLinks(): Promise<void> {
  if (!currentUri) return;
  const res = await send<{ ok: boolean; data?: Array<{ uri_a: string; uri_b: string; note?: string }>; error?: string }>({
    type: "listLinks",
    payload: { uri: currentUri },
  });
  if (!res.ok || !res.data) {
    elLinksList.innerHTML = `<li class="empty">${res.error ?? "Error loading links"}</li>`;
    return;
  }
  if (res.data.length === 0) {
    elLinksList.innerHTML = '<li class="empty">No links yet.</li>';
    return;
  }
  elLinksList.innerHTML = res.data
    .map((l) => {
      const other = l.uri_a === currentUri ? l.uri_b : l.uri_a;
      const note = l.note ? ` — ${l.note}` : "";
      return `<li title="${other}${note}">${other}${note}</li>`;
    })
    .join("");
}

document.getElementById("btn-copy-uri")!.addEventListener("click", async () => {
  if (!currentUri) return;
  await navigator.clipboard.writeText(currentUri);
  setStatus("✓ Copied to clipboard");
});

document.getElementById("btn-create-link")!.addEventListener("click", async () => {
  const target = elLinkTarget.value.trim();
  if (!target) { setStatus("Enter a target URI first."); return; }
  if (!currentUri) { setStatus("No page URI available."); return; }

  const res = await send<{ ok: boolean; error?: string }>({
    type: "createLink",
    payload: { uriA: currentUri, uriB: target, note: elLinkNote.value.trim() || undefined },
  });
  if (res.ok) {
    elLinkTarget.value = "";
    elLinkNote.value = "";
    setStatus("✓ Link created");
    await loadLinks();
  } else {
    setStatus(`Error: ${res.error}`);
  }
});

document.getElementById("btn-save-settings")!.addEventListener("click", async () => {
  const url = elServerUrl.value.trim();
  if (!url) return;
  await send({ type: "updateSettings", payload: { serverUrl: url } });
  setStatus("✓ Settings saved — reopen popup to reconnect");
});

init().catch((e) => setStatus(`Error: ${(e as Error).message}`));

/**
 * Hookmarks OneNote Add-in — task pane entry point.
 *
 * Renders a simple UI in the Office task pane panel:
 *   - Shows current page's hook:// URI
 *   - Lists existing links
 *   - Lets user create/delete links
 */

import { HKAddInBridge } from "./bridge";

let bridge: HKAddInBridge;
let currentPageUri = "";

Office.onReady(({ host }) => {
  if (host === Office.HostType.OneNote) {
    bridge = new HKAddInBridge(
      (document.getElementById("serverUrl") as HTMLInputElement)?.value ||
        "http://127.0.0.1:2701"
    );
    initUI();
  }
});

// ── UI init ───────────────────────────────────────────────────────────────────

function initUI(): void {
  document.getElementById("btn-refresh")?.addEventListener("click", loadCurrentPage);
  document.getElementById("btn-create-link")?.addEventListener("click", createLink);
  document.getElementById("btn-copy-uri")?.addEventListener("click", copyUri);
  document.getElementById("server-url-input")?.addEventListener("change", (e) => {
    bridge = new HKAddInBridge((e.target as HTMLInputElement).value);
  });

  loadCurrentPage();
}

async function loadCurrentPage(): Promise<void> {
  setStatus("Loading…");

  try {
    await OneNote.run(async (context) => {
      const page = context.application.getActivePage();
      page.load("webUrl,title");
      await context.sync();

      currentPageUri = HKAddInBridge.buildPageUri(page.webUrl);

      const uriEl = document.getElementById("current-uri");
      const titleEl = document.getElementById("page-title");
      if (uriEl) uriEl.textContent = currentPageUri;
      if (titleEl) titleEl.textContent = page.title;
    });
  } catch (e) {
    setStatus(`Error reading page: ${e}`);
    return;
  }

  await loadLinks();
}

async function loadLinks(): Promise<void> {
  if (!currentPageUri) return;

  const reachable = await bridge.probeServer();
  if (!reachable) {
    setStatus("⚠️ hk serve is not running. Start it with: hk serve");
    return;
  }

  const result = await bridge.listLinks(currentPageUri);
  if (!result.ok) {
    setStatus(`Error: ${result.error}`);
    return;
  }

  renderLinks(result.value ?? []);
  setStatus(`✓ ${result.value?.length ?? 0} link(s)`);
}

function renderLinks(links: import("./bridge").LinkRecord[]): void {
  const list = document.getElementById("links-list");
  if (!list) return;

  list.innerHTML = "";
  if (links.length === 0) {
    list.innerHTML = "<li class='empty'>No links yet.</li>";
    return;
  }

  for (const link of links) {
    const other = link.target === currentPageUri ? link.source : link.target;
    const li = document.createElement("li");
    li.innerHTML = `
      <span class="link-uri" title="${other}">${truncate(other, 50)}</span>
      ${link.note ? `<span class="link-note">${link.note}</span>` : ""}
      <button class="btn-delete" data-a="${currentPageUri}" data-b="${other}">✕</button>
    `;
    li.querySelector(".btn-delete")?.addEventListener("click", async (e) => {
      const btn = e.target as HTMLButtonElement;
      await bridge.deleteLink(btn.dataset.a!, btn.dataset.b!);
      await loadLinks();
    });
    list.appendChild(li);
  }
}

async function createLink(): Promise<void> {
  const otherUri = (document.getElementById("link-target") as HTMLInputElement)?.value?.trim();
  const note = (document.getElementById("link-note") as HTMLInputElement)?.value?.trim();

  if (!otherUri || !currentPageUri) return;

  setStatus("Creating link…");
  const result = await bridge.createLink(currentPageUri, otherUri, note || undefined);

  if (!result.ok) {
    setStatus(`Error: ${result.error}`);
    return;
  }

  (document.getElementById("link-target") as HTMLInputElement).value = "";
  (document.getElementById("link-note") as HTMLInputElement).value = "";
  await loadLinks();
}

async function copyUri(): Promise<void> {
  if (!currentPageUri) return;
  await navigator.clipboard.writeText(currentPageUri);
  setStatus("✓ Copied to clipboard");
}

// ── helpers ───────────────────────────────────────────────────────────────────

function setStatus(msg: string): void {
  const el = document.getElementById("status");
  if (el) el.textContent = msg;
}

function truncate(s: string, n: number): string {
  return s.length > n ? s.slice(0, n) + "…" : s;
}

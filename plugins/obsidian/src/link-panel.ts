/**
 * Link Panel — a sidebar ItemView showing all hook:// links for the active note.
 */

import {
  ItemView,
  TFile,
  WorkspaceLeaf,
  Menu,
  Notice,
} from "obsidian";
import type HitchmarkPlugin from "./main";
import type { LinkRecord } from "./types";

export const LINK_PANEL_VIEW_TYPE = "hitchmark-link-panel";

export class LinkPanelView extends ItemView {
  private plugin: HitchmarkPlugin;
  private currentFile: TFile | null = null;
  private links: LinkRecord[] = [];
  private isLoading = false;
  private error: string | null = null;

  constructor(leaf: WorkspaceLeaf, plugin: HitchmarkPlugin) {
    super(leaf);
    this.plugin = plugin;
  }

  getViewType(): string {
    return LINK_PANEL_VIEW_TYPE;
  }

  getDisplayText(): string {
    return "Hitchmark Links";
  }

  getIcon(): string {
    return "link";
  }

  async onOpen(): Promise<void> {
    this.renderEmpty();
    // Subscribe to active file changes
    this.registerEvent(
      this.app.workspace.on("active-leaf-change", () => this.onActiveLeafChange())
    );
    await this.onActiveLeafChange();
  }

  async onClose(): Promise<void> {
    /* nothing to clean up */
  }

  private async onActiveLeafChange(): Promise<void> {
    const file = this.app.workspace.getActiveFile();
    if (file === this.currentFile) return;
    this.currentFile = file;
    await this.refresh();
  }

  /** Public API for forcing a refresh */
  async refresh(): Promise<void> {
    if (!this.currentFile) {
      this.renderEmpty();
      return;
    }

    this.isLoading = true;
    this.error = null;
    this.renderLoading();

    // Get hook:// URI for the current file
    const vaultPath = (this.app.vault.adapter as { basePath?: string }).basePath;
    const fullPath = vaultPath
      ? `${vaultPath}/${this.currentFile.path}`
      : this.currentFile.path;

    const uriResult = await this.plugin.bridge.fileToUri(fullPath);
    if (!uriResult.ok) {
      this.isLoading = false;
      this.error = uriResult.error;
      this.renderError(uriResult.error);
      return;
    }

    const linksResult = await this.plugin.bridge.listLinks(uriResult.value);
    this.isLoading = false;

    if (!linksResult.ok) {
      this.error = linksResult.error;
      this.renderError(linksResult.error);
      return;
    }

    this.links = linksResult.value;
    this.renderLinks(uriResult.value);
  }

  // ----- Rendering -----

  private getContainer(): HTMLElement {
    return this.containerEl.children[1] as HTMLElement;
  }

  private renderEmpty(): void {
    const container = this.getContainer();
    container.empty();
    container.createEl("div", {
      cls: "hitchmark-panel-empty",
      text: "Open a file to see its links.",
    });
  }

  private renderLoading(): void {
    const container = this.getContainer();
    container.empty();
    const wrap = container.createEl("div", { cls: "hitchmark-panel-loading" });
    wrap.createEl("span", { text: "Loading links…" });
  }

  private renderError(msg: string): void {
    const container = this.getContainer();
    container.empty();
    const err = container.createEl("div", { cls: "hitchmark-panel-error" });
    err.createEl("p", { text: "⚠️ Could not load links" });
    err.createEl("p", { text: msg, cls: "hitchmark-panel-error-detail" });
    const btn = err.createEl("button", { text: "Retry" });
    btn.addEventListener("click", () => this.refresh());
  }

  private renderLinks(currentUri: string): void {
    const container = this.getContainer();
    container.empty();

    // Header
    const header = container.createEl("div", { cls: "hitchmark-panel-header" });
    header.createEl("span", {
      text: `${this.links.length} link${this.links.length !== 1 ? "s" : ""}`,
      cls: "hitchmark-panel-count",
    });
    const refreshBtn = header.createEl("button", {
      cls: "hitchmark-panel-refresh",
      attr: { "aria-label": "Refresh links" },
    });
    refreshBtn.textContent = "↺";
    refreshBtn.addEventListener("click", () => this.refresh());

    if (this.links.length === 0) {
      container.createEl("div", {
        cls: "hitchmark-panel-no-links",
        text: "No links yet for this file.",
      });
      return;
    }

    // Link list
    const list = container.createEl("ul", { cls: "hitchmark-panel-list" });
    for (const link of this.links) {
      this.renderLinkItem(list, link, currentUri);
    }
  }

  private renderLinkItem(
    list: HTMLElement,
    link: LinkRecord,
    currentUri: string
  ): void {
    const li = list.createEl("li", { cls: "hitchmark-panel-item" });

    // Determine the "other" URI (not the current file)
    const otherUri =
      link.source === currentUri ? link.target : link.source;

    // Display label: use file path portion of hook:// URI
    const label = this.uriToLabel(otherUri);

    const itemContent = li.createEl("div", { cls: "hitchmark-panel-item-content" });

    const uriEl = itemContent.createEl("span", {
      cls: "hitchmark-panel-item-uri",
      text: label,
      attr: { title: otherUri },
    });

    uriEl.addEventListener("click", async () => {
      const result = await this.plugin.bridge.openUri(otherUri);
      if (!result.ok) new Notice(`Failed to open: ${result.error}`);
    });

    if (link.note) {
      itemContent.createEl("span", {
        cls: "hitchmark-panel-item-note",
        text: link.note,
      });
    }

    // Context menu
    li.addEventListener("contextmenu", (e) => {
      const menu = new Menu();
      menu.addItem((item) => {
        item
          .setTitle("Copy URI")
          .setIcon("copy")
          .onClick(() => {
            navigator.clipboard.writeText(otherUri);
            new Notice("Copied hook:// URI");
          });
      });
      menu.addItem((item) => {
        item
          .setTitle("Open link")
          .setIcon("external-link")
          .onClick(async () => {
            const result = await this.plugin.bridge.openUri(otherUri);
            if (!result.ok) new Notice(`Failed to open: ${result.error}`);
          });
      });
      menu.showAtMouseEvent(e);
    });
  }

  private uriToLabel(uri: string): string {
    // hook://file/<base64path>#para-<id> → decoded filename
    try {
      const match = uri.match(/^hook:\/\/file\/([^#]+)/);
      if (match) {
        const decoded = atob(match[1].replace(/-/g, "+").replace(/_/g, "/"));
        const parts = decoded.split("/");
        return parts[parts.length - 1] || decoded;
      }
    } catch {
      /* fall through */
    }
    return uri;
  }
}

/**
 * Hookmarks Obsidian Plugin — main entry point.
 *
 * Features:
 *  • Purple numbers in live editor (§abc123 annotations)
 *  • Commands: copy hook:// URI for note / paragraph / selection
 *  • Link panel: sidebar showing all bidirectional links for the active note
 *  • Settings: color, toggle, binary path
 */

import {
  Editor,
  MarkdownView,
  Notice,
  Plugin,
  TFile,
  WorkspaceLeaf,
} from "obsidian";
import { Extension } from "@codemirror/state";
import { DEFAULT_SETTINGS, HookmarksSettings } from "./types";
import { HKBridge } from "./bridge";
import { HookmarksSettingsTab } from "./settings";
import {
  LINK_PANEL_VIEW_TYPE,
  LinkPanelView,
} from "./link-panel";
import {
  fileUriField,
  purpleNumberExtension,
  setFileUriEffect,
} from "./purple-widget";
import { generatePurpleIdWithCollision, splitParagraphs } from "./purple";

export default class HookmarksPlugin extends Plugin {
  settings!: HookmarksSettings;
  bridge!: HKBridge;
  private cmExtensions: Extension[] = [];
  private purpleExtension: Extension | undefined = undefined;

  async onload(): Promise<void> {
    await this.loadSettings();
    this.bridge = new HKBridge(this.settings.hkBinaryPath);

    // Register link panel view
    this.registerView(
      LINK_PANEL_VIEW_TYPE,
      (leaf) => new LinkPanelView(leaf, this)
    );

    // Register CodeMirror extensions
    this.setupEditorExtensions();

    // Register commands
    this.registerCommands();

    // Settings tab
    this.addSettingTab(new HookmarksSettingsTab(this.app, this));

    // Ribbon icon to open link panel
    this.addRibbonIcon("link", "Hookmarks: Open link panel", () =>
      this.openLinkPanel()
    );

    // Update file URI in CM state when active file changes
    this.registerEvent(
      this.app.workspace.on("active-leaf-change", () =>
        this.updateEditorFileUri()
      )
    );

    // Also update on file open
    this.registerEvent(
      this.app.workspace.on("file-open", () => this.updateEditorFileUri())
    );

    console.log("Hookmarks plugin loaded");
  }

  async onunload(): Promise<void> {
    this.app.workspace.detachLeavesOfType(LINK_PANEL_VIEW_TYPE);
    console.log("Hookmarks plugin unloaded");
  }

  // ----- Settings -----

  async loadSettings(): Promise<void> {
    this.settings = Object.assign(
      {},
      DEFAULT_SETTINGS,
      await this.loadData()
    );
  }

  async saveSettings(): Promise<void> {
    await this.saveData(this.settings);
  }

  // ----- Editor Extensions -----

  private setupEditorExtensions(): void {
    const extensions: Extension[] = [fileUriField];

    if (this.settings.showPurpleNumbers) {
      const ext = purpleNumberExtension({
        onCopy: (uri) => this.copyToClipboard(uri),
      });
      this.purpleExtension = ext as unknown as Extension;
      extensions.push(this.purpleExtension);
    }

    this.cmExtensions = extensions;
    this.registerEditorExtension(this.cmExtensions);
  }

  togglePurpleNumbers(enabled: boolean): void {
    // Re-register with/without purple extension
    // Obsidian doesn't support dynamic removal, so we log guidance
    new Notice(
      enabled
        ? "Purple numbers enabled — restart Obsidian to apply."
        : "Purple numbers disabled — restart Obsidian to apply."
    );
  }

  updatePurpleNumberColor(color: string): void {
    document.documentElement.style.setProperty(
      "--hookmarks-purple-color",
      color
    );
  }

  private async updateEditorFileUri(): Promise<void> {
    const file = this.app.workspace.getActiveFile();
    if (!file) return;

    const vaultPath = (this.app.vault.adapter as { basePath?: string })
      .basePath;
    const fullPath = vaultPath ? `${vaultPath}/${file.path}` : file.path;

    const result = await this.bridge.fileToUri(fullPath);
    if (!result.ok) return;

    // Dispatch the file URI into all open CodeMirror editors for this file
    this.app.workspace.iterateRootLeaves((leaf) => {
      if (
        leaf.view instanceof MarkdownView &&
        leaf.view.file?.path === file.path
      ) {
        const editor = leaf.view.editor;
        const cmView = (editor as unknown as { cm?: { dispatch: (tr: unknown) => void } }).cm;
        if (cmView) {
          cmView.dispatch({
            effects: setFileUriEffect.of(result.ok ? result.value : ""),
          });
        }
      }
    });
  }

  // ----- Commands -----

  private registerCommands(): void {
    // Copy hook:// URI for the active note
    this.addCommand({
      id: "copy-file-uri",
      name: "Copy hook:// URI for active note",
      checkCallback: (checking) => {
        const file = this.app.workspace.getActiveFile();
        if (!file) return false;
        if (!checking) this.copyFileUri(file);
        return true;
      },
    });

    // Copy hook:// URI for cursor paragraph
    this.addCommand({
      id: "copy-paragraph-uri",
      name: "Copy hook:// URI for current paragraph",
      editorCheckCallback: (checking, editor) => {
        const file = this.app.workspace.getActiveFile();
        if (!file) return false;
        if (!checking) this.copyParagraphUri(editor, file);
        return true;
      },
    });

    // Open link panel
    this.addCommand({
      id: "open-link-panel",
      name: "Open linked documents panel",
      callback: () => this.openLinkPanel(),
    });

    // Create link between two URIs
    this.addCommand({
      id: "create-link",
      name: "Create link: active note ↔ clipboard URI",
      checkCallback: (checking) => {
        const file = this.app.workspace.getActiveFile();
        if (!file) return false;
        if (!checking) this.createLinkFromClipboard(file);
        return true;
      },
    });

    // Refresh link panel
    this.addCommand({
      id: "refresh-link-panel",
      name: "Refresh link panel",
      callback: () => {
        const leaves = this.app.workspace.getLeavesOfType(
          LINK_PANEL_VIEW_TYPE
        );
        for (const leaf of leaves) {
          if (leaf.view instanceof LinkPanelView) {
            leaf.view.refresh();
          }
        }
      },
    });
  }

  // ----- Command implementations -----

  private async copyFileUri(file: TFile): Promise<void> {
    const vaultPath = (this.app.vault.adapter as { basePath?: string })
      .basePath;
    const fullPath = vaultPath ? `${vaultPath}/${file.path}` : file.path;

    const result = await this.bridge.fileToUri(fullPath);
    if (!result.ok) {
      new Notice(`Failed to get URI: ${result.error}`);
      return;
    }
    await this.copyToClipboard(result.value);
    new Notice(`Copied: ${result.value}`);
  }

  private async copyParagraphUri(editor: Editor, file: TFile): Promise<void> {
    const cursor = editor.getCursor();
    const fullText = editor.getValue();
    const paragraphs = splitParagraphs(fullText);

    // Find which paragraph the cursor is in
    let offset = 0;
    let paragraphText: string | null = null;

    for (const para of fullText.split(/\n\n+/)) {
      const trimmed = para.trim();
      if (!trimmed) {
        offset += para.length + 2;
        continue;
      }
      const paraEnd = offset + para.length;
      const cursorOffset = editor.posToOffset(cursor);
      if (cursorOffset >= offset && cursorOffset <= paraEnd) {
        paragraphText = trimmed;
        break;
      }
      offset = paraEnd + 2;
    }

    if (!paragraphText) {
      new Notice("No paragraph found at cursor position.");
      return;
    }

    // Compute purple ID
    const seen = new Set<string>();
    let paraId = "";
    for (const para of paragraphs) {
      const id = generatePurpleIdWithCollision(para, seen);
      if (para === paragraphText) {
        paraId = id;
        break;
      }
    }

    if (!paraId) {
      new Notice("Could not compute paragraph ID.");
      return;
    }

    // Get the file URI
    const vaultPath = (this.app.vault.adapter as { basePath?: string })
      .basePath;
    const fullPath = vaultPath ? `${vaultPath}/${file.path}` : file.path;
    const fileResult = await this.bridge.fileToUri(fullPath);

    const uri = fileResult.ok
      ? `${fileResult.value}#para-${paraId}`
      : `#para-${paraId}`;

    await this.copyToClipboard(uri);
    new Notice(`Copied: ${uri}`);
  }

  private async createLinkFromClipboard(file: TFile): Promise<void> {
    const clipText = await navigator.clipboard.readText();
    if (!clipText.startsWith("hook://")) {
      new Notice("Clipboard doesn't contain a hook:// URI.");
      return;
    }

    const vaultPath = (this.app.vault.adapter as { basePath?: string })
      .basePath;
    const fullPath = vaultPath ? `${vaultPath}/${file.path}` : file.path;
    const fileResult = await this.bridge.fileToUri(fullPath);

    if (!fileResult.ok) {
      new Notice(`Failed to get file URI: ${fileResult.error}`);
      return;
    }

    const result = await this.bridge.createLink(
      fileResult.value,
      clipText.trim()
    );
    if (!result.ok) {
      new Notice(`Failed to create link: ${result.error}`);
    } else {
      new Notice("✅ Link created");
      // Refresh any open link panels
      const leaves = this.app.workspace.getLeavesOfType(LINK_PANEL_VIEW_TYPE);
      for (const leaf of leaves) {
        if (leaf.view instanceof LinkPanelView) leaf.view.refresh();
      }
    }
  }

  // ----- Link Panel -----

  async openLinkPanel(): Promise<void> {
    const existing = this.app.workspace.getLeavesOfType(LINK_PANEL_VIEW_TYPE);
    if (existing.length > 0) {
      this.app.workspace.revealLeaf(existing[0]);
      return;
    }

    const leaf = this.app.workspace.getRightLeaf(false);
    if (!leaf) return;
    await leaf.setViewState({ type: LINK_PANEL_VIEW_TYPE, active: true });
    this.app.workspace.revealLeaf(leaf);
  }

  // ----- Utilities -----

  private async copyToClipboard(text: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      // Fallback for environments where clipboard API is restricted
      const el = document.createElement("textarea");
      el.value = text;
      document.body.appendChild(el);
      el.select();
      document.execCommand("copy");
      document.body.removeChild(el);
    }
  }
}

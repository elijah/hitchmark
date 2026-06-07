/**
 * Plugin settings tab — shown in Obsidian's Settings panel.
 */

import { App, Notice, PluginSettingTab, Setting } from "obsidian";
import type HookmarksPlugin from "./main";

export class HookmarksSettingsTab extends PluginSettingTab {
  plugin: HookmarksPlugin;

  constructor(app: App, plugin: HookmarksPlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  display(): void {
    const { containerEl } = this;
    containerEl.empty();
    containerEl.createEl("h2", { text: "Hookmarks" });

    // ----- Purple Numbers -----
    containerEl.createEl("h3", { text: "Purple Numbers" });

    new Setting(containerEl)
      .setName("Show purple numbers")
      .setDesc("Display §id annotations beside paragraphs in the live editor.")
      .addToggle((toggle) =>
        toggle
          .setValue(this.plugin.settings.showPurpleNumbers)
          .onChange(async (value) => {
            this.plugin.settings.showPurpleNumbers = value;
            await this.plugin.saveSettings();
            this.plugin.togglePurpleNumbers(value);
          })
      );

    new Setting(containerEl)
      .setName("Annotation color")
      .setDesc("CSS color for purple number annotations.")
      .addColorPicker((cp) =>
        cp
          .setValue(this.plugin.settings.purpleNumberColor)
          .onChange(async (value) => {
            this.plugin.settings.purpleNumberColor = value;
            await this.plugin.saveSettings();
            this.plugin.updatePurpleNumberColor(value);
          })
      )
      .addText((text) =>
        text
          .setValue(this.plugin.settings.purpleNumberColor)
          .onChange(async (value) => {
            this.plugin.settings.purpleNumberColor = value;
            await this.plugin.saveSettings();
            this.plugin.updatePurpleNumberColor(value);
          })
      );

    new Setting(containerEl)
      .setName("Copy URI on click")
      .setDesc("Clicking a §id annotation copies the hook:// paragraph URI to clipboard.")
      .addToggle((toggle) =>
        toggle
          .setValue(this.plugin.settings.copyOnClick)
          .onChange(async (value) => {
            this.plugin.settings.copyOnClick = value;
            await this.plugin.saveSettings();
          })
      );

    // ----- CLI Integration -----
    containerEl.createEl("h3", { text: "CLI Integration" });

    new Setting(containerEl)
      .setName("Path to hk binary")
      .setDesc(
        "Absolute path to the hk CLI. Leave blank to auto-detect in: " +
          "/usr/local/bin, ~/.cargo/bin, /opt/homebrew/bin"
      )
      .addText((text) =>
        text
          .setPlaceholder("/usr/local/bin/hk")
          .setValue(this.plugin.settings.hkBinaryPath)
          .onChange(async (value) => {
            this.plugin.settings.hkBinaryPath = value.trim();
            await this.plugin.saveSettings();
            this.plugin.bridge.invalidateCache();
          })
      );

    new Setting(containerEl)
      .setName("Test connection")
      .setDesc("Verify that the hk binary is reachable.")
      .addButton((btn) =>
        btn.setButtonText("Test").onClick(async () => {
          btn.setButtonText("Testing…");
          const ok = await this.plugin.bridge.ping();
          btn.setButtonText("Test");
          if (ok) {
            new Notice("✅ hk is reachable");
          } else {
            new Notice(
              "❌ hk not found. Install it with: cargo install hookmarks-cli"
            );
          }
        })
      );

    // ----- Advanced -----
    containerEl.createEl("h3", { text: "Advanced" });

    new Setting(containerEl)
      .setName("Daemon URL")
      .setDesc(
        "URL for the local hk HTTP server (hk serve). " +
          "Optional — only needed for web-based vault access."
      )
      .addText((text) =>
        text
          .setPlaceholder("http://localhost:7878")
          .setValue(this.plugin.settings.daemonUrl)
          .onChange(async (value) => {
            this.plugin.settings.daemonUrl = value.trim();
            await this.plugin.saveSettings();
          })
      );

    // ----- Info -----
    containerEl.createEl("h3", { text: "About" });
    const info = containerEl.createEl("div", { cls: "hookmarks-settings-info" });
    info.createEl("p", {
      text: "Hookmarks: stable, addressable links to documents and paragraphs.",
    });
    const link = info.createEl("a", {
      text: "GitHub: not-hookmarks",
      href: "https://github.com/elw/not-hookmarks",
    });
    link.setAttribute("target", "_blank");
  }
}

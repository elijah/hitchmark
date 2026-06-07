// Minimal Obsidian API mock for Jest tests
export class Plugin {}
export class ItemView {}
export class PluginSettingTab {}
export class Setting {
  setName(_: string) { return this; }
  setDesc(_: string) { return this; }
  addToggle(_: (t: { setValue: (v: boolean) => typeof this; onChange: (cb: (v: boolean) => Promise<void>) => typeof this }) => void) { return this; }
  addText(_: (t: unknown) => void) { return this; }
  addButton(_: (b: unknown) => void) { return this; }
  addColorPicker(_: (cp: unknown) => void) { return this; }
}
export class Notice {
  constructor(msg: string) { void msg; }
}
export class Menu {
  addItem(_: (item: unknown) => void) { return this; }
  showAtMouseEvent(_: MouseEvent) {}
}
export class TFile {}
export class MarkdownView {}
export class WorkspaceLeaf {}

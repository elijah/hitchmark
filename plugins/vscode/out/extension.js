"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/extension.ts
var extension_exports = {};
__export(extension_exports, {
  activate: () => activate,
  deactivate: () => deactivate
});
module.exports = __toCommonJS(extension_exports);
var vscode = __toESM(require("vscode"));

// src/bridge.ts
var import_child_process = require("child_process");
var import_util = require("util");
var fs = __toESM(require("fs"));
var execFileAsync = (0, import_util.promisify)(import_child_process.execFile);
var HK_SEARCH_PATHS = [
  "/usr/local/bin/hk",
  `${process.env.HOME ?? ""}/.cargo/bin/hk`,
  "/opt/homebrew/bin/hk",
  "/usr/bin/hk"
];
var HKBridge = class {
  constructor(cliPath = "", serverUrl = "http://127.0.0.1:2701") {
    this.cliPath = cliPath;
    this.serverUrl = serverUrl;
    this.resolvedPath = null;
    this.serverReachable = null;
  }
  invalidateCache() {
    this.resolvedPath = null;
    this.serverReachable = null;
  }
  resolvePath() {
    if (this.resolvedPath)
      return this.resolvedPath;
    if (this.cliPath && fs.existsSync(this.cliPath)) {
      this.resolvedPath = this.cliPath;
      return this.resolvedPath;
    }
    for (const p of HK_SEARCH_PATHS) {
      if (fs.existsSync(p)) {
        this.resolvedPath = p;
        return p;
      }
    }
    return null;
  }
  async probeServer() {
    if (!this.serverUrl)
      return false;
    if (this.serverReachable !== null)
      return this.serverReachable;
    try {
      const res = await fetch(`${this.serverUrl}/health`, {
        signal: AbortSignal.timeout(1500)
      });
      this.serverReachable = res.ok;
    } catch {
      this.serverReachable = false;
    }
    return this.serverReachable;
  }
  async httpGet(path) {
    try {
      const res = await fetch(`${this.serverUrl}${path}`, {
        signal: AbortSignal.timeout(5e3)
      });
      const json = await res.json();
      if (!res.ok)
        return { ok: false, error: json.error };
      return { ok: true, value: json };
    } catch (e) {
      this.serverReachable = null;
      return { ok: false, error: String(e) };
    }
  }
  async httpPost(path, body) {
    try {
      const res = await fetch(`${this.serverUrl}${path}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
        signal: AbortSignal.timeout(5e3)
      });
      const json = await res.json();
      if (!res.ok)
        return { ok: false, error: json.error };
      return { ok: true, value: json };
    } catch (e) {
      this.serverReachable = null;
      return { ok: false, error: String(e) };
    }
  }
  async run(subcommand, args) {
    const hkPath = this.resolvePath();
    if (!hkPath) {
      return {
        ok: false,
        error: "hk binary not found. Install with: cargo install hookmarks-cli"
      };
    }
    try {
      const { stdout, stderr } = await execFileAsync(hkPath, [subcommand, ...args], {
        timeout: 1e4
      });
      if (stderr.trim())
        return { ok: false, error: stderr.trim() };
      return { ok: true, value: stdout.trim() };
    } catch (e) {
      return { ok: false, error: String(e) };
    }
  }
  async fileToUri(filePath) {
    if (await this.probeServer()) {
      const r = await this.httpGet(
        `/uri?path=${encodeURIComponent(filePath)}`
      );
      if (r.ok && r.value)
        return { ok: true, value: r.value.uri };
    }
    return this.run("file", [filePath]);
  }
  async listLinks(uri) {
    if (await this.probeServer()) {
      return this.httpGet(`/links?uri=${encodeURIComponent(uri)}`);
    }
    const r = await this.run("list", [uri, "--json"]);
    if (!r.ok)
      return { ok: false, error: r.error };
    try {
      return { ok: true, value: JSON.parse(r.value ?? "[]") };
    } catch {
      return { ok: false, error: "Failed to parse JSON" };
    }
  }
  async createLink(uriA, uriB, note) {
    if (await this.probeServer()) {
      const r2 = await this.httpPost("/links", {
        uri_a: uriA,
        uri_b: uriB,
        note
      });
      return r2.ok ? { ok: true } : { ok: false, error: r2.error };
    }
    const args = note ? [uriA, uriB, "--note", note] : [uriA, uriB];
    const r = await this.run("link", args);
    return r.ok ? { ok: true } : { ok: false, error: r.error };
  }
  async getPurpleNumbers(filePath) {
    if (await this.probeServer()) {
      return this.httpGet(
        `/purple?path=${encodeURIComponent(filePath)}`
      );
    }
    const r = await this.run("purple", [filePath, "--format", "json"]);
    if (!r.ok)
      return { ok: false, error: r.error };
    try {
      return { ok: true, value: JSON.parse(r.value ?? "[]") };
    } catch {
      return { ok: false, error: "Failed to parse JSON" };
    }
  }
  async openUri(uri) {
    const r = await this.run("open", [uri]);
    return r.ok ? { ok: true } : { ok: false, error: r.error };
  }
};

// src/extension.ts
var bridge;
function activate(context) {
  bridge = buildBridge();
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("hookmarks")) {
        bridge = buildBridge();
      }
    })
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("hookmarks.copyUri", cmdCopyUri),
    vscode.commands.registerCommand("hookmarks.copyUriWithParagraph", cmdCopyUriWithParagraph),
    vscode.commands.registerCommand("hookmarks.listLinks", cmdListLinks),
    vscode.commands.registerCommand("hookmarks.openUri", cmdOpenUri),
    vscode.commands.registerCommand("hookmarks.showPurpleNumbers", cmdShowPurpleNumbers),
    vscode.commands.registerCommand("hookmarks.startServer", cmdStartServer)
  );
}
function deactivate() {
}
function buildBridge() {
  const cfg = vscode.workspace.getConfiguration("hookmarks");
  const cliPath = cfg.get("cliPath", "");
  const serverUrl = cfg.get("serverUrl", "http://127.0.0.1:2701");
  return new HKBridge(cliPath, serverUrl);
}
async function cmdCopyUri(_uri) {
  const filePath = resolveFilePath(_uri);
  if (!filePath)
    return;
  const result = await bridge.fileToUri(filePath);
  if (!result.ok || !result.value) {
    vscode.window.showErrorMessage(`Hookmarks: ${result.error}`);
    return;
  }
  await vscode.env.clipboard.writeText(result.value);
  vscode.window.showInformationMessage(`Copied: ${result.value}`);
}
async function cmdCopyUriWithParagraph() {
  const editor = vscode.window.activeTextEditor;
  if (!editor)
    return;
  const filePath = editor.document.uri.fsPath;
  const cursorLine = editor.selection.active.line;
  const uriResult = await bridge.fileToUri(filePath);
  if (!uriResult.ok || !uriResult.value) {
    vscode.window.showErrorMessage(`Hookmarks: ${uriResult.error}`);
    return;
  }
  const purpleResult = await bridge.getPurpleNumbers(filePath);
  if (!purpleResult.ok || !purpleResult.value?.length) {
    await vscode.env.clipboard.writeText(uriResult.value);
    vscode.window.showInformationMessage(`Copied (no paragraph IDs): ${uriResult.value}`);
    return;
  }
  const paraId = findNearestParagraph(editor.document, cursorLine, purpleResult.value);
  const fullUri = paraId ? `${uriResult.value}#para-${paraId}` : uriResult.value;
  await vscode.env.clipboard.writeText(fullUri);
  vscode.window.showInformationMessage(`Copied: ${fullUri}`);
}
async function cmdListLinks() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showWarningMessage("Hookmarks: Open a file first.");
    return;
  }
  const uriResult = await bridge.fileToUri(editor.document.uri.fsPath);
  if (!uriResult.ok || !uriResult.value) {
    vscode.window.showErrorMessage(`Hookmarks: ${uriResult.error}`);
    return;
  }
  const linksResult = await bridge.listLinks(uriResult.value);
  if (!linksResult.ok) {
    vscode.window.showErrorMessage(`Hookmarks: ${linksResult.error}`);
    return;
  }
  const links = linksResult.value ?? [];
  if (links.length === 0) {
    vscode.window.showInformationMessage("Hookmarks: No links for this file.");
    return;
  }
  const items = links.map((l) => ({
    label: l.target === uriResult.value ? l.source : l.target,
    description: l.note ?? "",
    detail: l.created_at,
    uri: l.target === uriResult.value ? l.source : l.target
  }));
  const picked = await vscode.window.showQuickPick(items, {
    placeHolder: `${links.length} link(s) for ${editor.document.fileName}`
  });
  if (picked) {
    await vscode.env.clipboard.writeText(picked.uri);
    vscode.window.showInformationMessage(`Copied: ${picked.uri}`);
  }
}
async function cmdOpenUri() {
  const clipboard = await vscode.env.clipboard.readText();
  const initial = clipboard.startsWith("hook://") ? clipboard : "";
  const input = await vscode.window.showInputBox({
    prompt: "Enter hook:// URI to open",
    value: initial,
    validateInput: (v) => v.startsWith("hook://") ? null : "Must be a hook:// URI"
  });
  if (!input)
    return;
  const result = await bridge.openUri(input);
  if (!result.ok) {
    vscode.window.showErrorMessage(`Hookmarks: ${result.error}`);
  }
}
async function cmdShowPurpleNumbers() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showWarningMessage("Hookmarks: Open a file first.");
    return;
  }
  const result = await bridge.getPurpleNumbers(editor.document.uri.fsPath);
  if (!result.ok) {
    vscode.window.showErrorMessage(`Hookmarks: ${result.error}`);
    return;
  }
  const items = (result.value ?? []).map((p) => ({
    label: `\xB6 ${p.id}`,
    description: p.text.slice(0, 80) + (p.text.length > 80 ? "\u2026" : ""),
    id: p.id
  }));
  if (items.length === 0) {
    vscode.window.showInformationMessage("Hookmarks: No paragraphs found.");
    return;
  }
  const picked = await vscode.window.showQuickPick(items, {
    placeHolder: "Select a paragraph to copy its ID"
  });
  if (picked) {
    await vscode.env.clipboard.writeText(`para-${picked.id}`);
    vscode.window.showInformationMessage(`Copied: para-${picked.id}`);
  }
}
async function cmdStartServer() {
  const terminal = vscode.window.createTerminal("hk serve");
  terminal.show();
  terminal.sendText("hk serve");
  vscode.window.showInformationMessage("Hookmarks server starting at http://127.0.0.1:2701");
}
function resolveFilePath(uri) {
  if (uri)
    return uri.fsPath;
  return vscode.window.activeTextEditor?.document.uri.fsPath;
}
function findNearestParagraph(doc, cursorLine, paragraphs) {
  let paraText = "";
  for (let i = cursorLine; i >= 0; i--) {
    const line = doc.lineAt(i).text;
    if (line.trim() === "" && i !== cursorLine)
      break;
    paraText = line + (paraText ? "\n" + paraText : "");
  }
  paraText = paraText.trim();
  const match = paragraphs.find(
    (p) => paraText.includes(p.text.slice(0, 30))
  );
  return match?.id;
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  activate,
  deactivate
});
//# sourceMappingURL=extension.js.map

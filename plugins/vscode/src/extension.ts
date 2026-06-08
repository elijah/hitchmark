/**
 * Hookmarks VS Code Extension — entry point.
 *
 * Registers 6 commands, reads config, and wires HKBridge.
 */

import * as vscode from "vscode";
import { HKBridge } from "./bridge";
import type { LinkRecord, PurpleRecord } from "./bridge";

let bridge: HKBridge;

export function activate(context: vscode.ExtensionContext): void {
  bridge = buildBridge();

  // Re-build bridge when settings change
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

export function deactivate(): void {}

// ── bridge factory ────────────────────────────────────────────────────────────

function buildBridge(): HKBridge {
  const cfg = vscode.workspace.getConfiguration("hookmarks");
  const cliPath = cfg.get<string>("cliPath", "");
  const serverUrl = cfg.get<string>("serverUrl", "http://127.0.0.1:2701");
  return new HKBridge(cliPath, serverUrl);
}

// ── commands ──────────────────────────────────────────────────────────────────

async function cmdCopyUri(_uri?: vscode.Uri): Promise<void> {
  const filePath = resolveFilePath(_uri);
  if (!filePath) return;

  const result = await bridge.fileToUri(filePath);
  if (!result.ok || !result.value) {
    vscode.window.showErrorMessage(`Hookmarks: ${result.error}`);
    return;
  }
  await vscode.env.clipboard.writeText(result.value);
  vscode.window.showInformationMessage(`Copied: ${result.value}`);
}

async function cmdCopyUriWithParagraph(): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;

  const filePath = editor.document.uri.fsPath;
  const cursorLine = editor.selection.active.line;

  // Get the file URI
  const uriResult = await bridge.fileToUri(filePath);
  if (!uriResult.ok || !uriResult.value) {
    vscode.window.showErrorMessage(`Hookmarks: ${uriResult.error}`);
    return;
  }

  // Get purple numbers and find the nearest one to cursor
  const purpleResult = await bridge.getPurpleNumbers(filePath);
  if (!purpleResult.ok || !purpleResult.value?.length) {
    // Fall back to plain URI
    await vscode.env.clipboard.writeText(uriResult.value);
    vscode.window.showInformationMessage(`Copied (no paragraph IDs): ${uriResult.value}`);
    return;
  }

  const paraId = findNearestParagraph(editor.document, cursorLine, purpleResult.value);
  const fullUri = paraId
    ? `${uriResult.value}#para-${paraId}`
    : uriResult.value;

  await vscode.env.clipboard.writeText(fullUri);
  vscode.window.showInformationMessage(`Copied: ${fullUri}`);
}

async function cmdListLinks(): Promise<void> {
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

  // Show as quick pick — clicking navigates to the other end
  const items = links.map((l: LinkRecord) => ({
    label: l.target === uriResult.value ? l.source : l.target,
    description: l.note ?? "",
    detail: l.created_at,
    uri: l.target === uriResult.value ? l.source : l.target,
  }));

  const picked = await vscode.window.showQuickPick(items, {
    placeHolder: `${links.length} link(s) for ${editor.document.fileName}`,
  });

  if (picked) {
    await vscode.env.clipboard.writeText(picked.uri);
    vscode.window.showInformationMessage(`Copied: ${picked.uri}`);
  }
}

async function cmdOpenUri(): Promise<void> {
  const clipboard = await vscode.env.clipboard.readText();
  const initial = clipboard.startsWith("hook://") ? clipboard : "";

  const input = await vscode.window.showInputBox({
    prompt: "Enter hook:// URI to open",
    value: initial,
    validateInput: (v) =>
      v.startsWith("hook://") ? null : "Must be a hook:// URI",
  });

  if (!input) return;

  const result = await bridge.openUri(input);
  if (!result.ok) {
    vscode.window.showErrorMessage(`Hookmarks: ${result.error}`);
  }
}

async function cmdShowPurpleNumbers(): Promise<void> {
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

  const items = (result.value ?? []).map((p: PurpleRecord) => ({
    label: `¶ ${p.id}`,
    description: p.text.slice(0, 80) + (p.text.length > 80 ? "…" : ""),
    id: p.id,
  }));

  if (items.length === 0) {
    vscode.window.showInformationMessage("Hookmarks: No paragraphs found.");
    return;
  }

  const picked = await vscode.window.showQuickPick(items, {
    placeHolder: "Select a paragraph to copy its ID",
  });

  if (picked) {
    await vscode.env.clipboard.writeText(`para-${picked.id}`);
    vscode.window.showInformationMessage(`Copied: para-${picked.id}`);
  }
}

async function cmdStartServer(): Promise<void> {
  const terminal = vscode.window.createTerminal("hk serve");
  terminal.show();
  terminal.sendText("hk serve");
  vscode.window.showInformationMessage("Hookmarks server starting at http://127.0.0.1:2701");
}

// ── helpers ───────────────────────────────────────────────────────────────────

function resolveFilePath(uri?: vscode.Uri): string | undefined {
  if (uri) return uri.fsPath;
  return vscode.window.activeTextEditor?.document.uri.fsPath;
}

function findNearestParagraph(
  doc: vscode.TextDocument,
  cursorLine: number,
  paragraphs: PurpleRecord[]
): string | undefined {
  // Find which paragraph the cursor is in by matching text content
  // Walk backwards from cursor line to find paragraph start
  let paraText = "";
  for (let i = cursorLine; i >= 0; i--) {
    const line = doc.lineAt(i).text;
    if (line.trim() === "" && i !== cursorLine) break;
    paraText = line + (paraText ? "\n" + paraText : "");
  }
  paraText = paraText.trim();

  const match = paragraphs.find(
    (p) => paraText.includes(p.text.slice(0, 30))
  );
  return match?.id;
}

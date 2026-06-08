/**
 * CodeMirror 6 extension that renders purple numbers in the editor.
 *
 * Adds a small `§abc123` annotation to the right of each paragraph
 * in Obsidian's live-preview editor. Clicking copies the hook:// URI
 * for that paragraph to the clipboard.
 */

import {
  Decoration,
  DecorationSet,
  EditorView,
  PluginValue,
  ViewPlugin,
  ViewUpdate,
  WidgetType,
} from "@codemirror/view";
import { RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import { generatePurpleIdWithCollision } from "./purple";

// ----- State effect for passing file URI into the extension -----

export const setFileUriEffect = StateEffect.define<string>();

export const fileUriField = StateField.define<string>({
  create: () => "",
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setFileUriEffect)) return effect.value;
    }
    return value;
  },
});

// ----- Purple number widget -----

class PurpleNumberWidget extends WidgetType {
  constructor(
    readonly id: string,
    readonly fileUri: string,
    readonly onCopy: (uri: string) => void
  ) {
    super();
  }

  toDOM(): HTMLElement {
    const el = document.createElement("span");
    el.className = "hitchmark-purple-number";
    el.textContent = `§${this.id}`;
    el.setAttribute("aria-label", `Purple number ${this.id} — click to copy hook URI`);
    el.setAttribute("role", "button");
    el.setAttribute("tabindex", "0");

    const fullUri = this.fileUri
      ? `${this.fileUri}#para-${this.id}`
      : `#para-${this.id}`;

    el.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.onCopy(fullUri);
    });

    el.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        this.onCopy(fullUri);
      }
    });

    return el;
  }

  /** Two widgets with the same id are equivalent */
  eq(other: PurpleNumberWidget): boolean {
    return other.id === this.id && other.fileUri === this.fileUri;
  }

  ignoreEvent(): boolean {
    return false;
  }
}

// ----- Helper: find paragraph end positions in the document -----

interface ParagraphBoundary {
  /** Position at the end of the last non-empty line in the paragraph */
  endPos: number;
  /** The full trimmed text of the paragraph */
  text: string;
}

function findParagraphBoundaries(
  view: EditorView
): ParagraphBoundary[] {
  const doc = view.state.doc;
  const text = doc.toString();
  const boundaries: ParagraphBoundary[] = [];

  // Suppress the unused paraRegex lint (kept for future regex-based impl)
  const paraRegex = /[^\n].*?(?=\n\n|\n$|$)/gs;

  // Collect paragraph spans with their end positions
  const lines = text.split("\n");
  let pos = 0;
  let paraStart: number | null = null;
  let paraLines: string[] = [];

  for (let i = 0; i <= lines.length; i++) {
    const line = i < lines.length ? lines[i] : "";
    const isEmpty = line.trim().length === 0;

    if (!isEmpty) {
      if (paraStart === null) paraStart = pos;
      paraLines.push(line);
    } else if (paraStart !== null && paraLines.length > 0) {
      // End of paragraph: record it
      const paraText = paraLines.join("\n").trim();
      const paraEndPos = pos - 1; // position after last char of paragraph
      boundaries.push({ endPos: paraEndPos, text: paraText });
      paraStart = null;
      paraLines = [];
    }

    pos += line.length + 1; // +1 for \n
  }

  // Handle final paragraph without trailing newline
  if (paraStart !== null && paraLines.length > 0) {
    const paraText = paraLines.join("\n").trim();
    boundaries.push({ endPos: pos - 1, text: paraText });
  }

  // Suppress unused variable warning
  void paraRegex;

  return boundaries;
}

// ----- View plugin -----

interface PurpleWidgetPluginOptions {
  onCopy: (uri: string) => void;
}

function buildDecorations(
  view: EditorView,
  onCopy: (uri: string) => void
): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  const fileUri = view.state.field(fileUriField, false) ?? "";
  const boundaries = findParagraphBoundaries(view);
  const seen = new Set<string>();

  for (const { endPos, text } of boundaries) {
    if (endPos < 0 || endPos > view.state.doc.length) continue;

    const id = generatePurpleIdWithCollision(text, seen);
    builder.add(
      endPos,
      endPos,
      Decoration.widget({
        widget: new PurpleNumberWidget(id, fileUri, onCopy),
        side: 1,
      })
    );
  }

  return builder.finish();
}

class PurpleWidgetPlugin implements PluginValue {
  decorations: DecorationSet;
  private onCopy: (uri: string) => void;

  constructor(view: EditorView, opts: PurpleWidgetPluginOptions) {
    this.onCopy = opts.onCopy;
    this.decorations = buildDecorations(view, this.onCopy);
  }

  update(update: ViewUpdate): void {
    if (
      update.docChanged ||
      update.viewportChanged ||
      update.transactions.some((tr) =>
        tr.effects.some((e) => e.is(setFileUriEffect))
      )
    ) {
      this.decorations = buildDecorations(update.view, this.onCopy);
    }
  }

  destroy(): void {
    /* nothing to clean up */
  }
}

/** Create the CodeMirror 6 extension for purple numbers */
export function purpleNumberExtension(opts: {
  onCopy: (uri: string) => void;
}): ReturnType<typeof ViewPlugin.define> {
  return ViewPlugin.define(
    (view) => new PurpleWidgetPlugin(view, opts),
    { decorations: (plugin) => plugin.decorations }
  );
}

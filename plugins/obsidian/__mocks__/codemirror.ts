// Minimal CodeMirror mock for Jest tests
export const Decoration = {
  widget: (_: unknown) => ({}),
  mark: (_: unknown) => ({}),
};
export class WidgetType {
  toDOM(): HTMLElement { return document.createElement("span"); }
  eq(_: WidgetType): boolean { return false; }
  ignoreEvent(): boolean { return false; }
}
export class ViewPlugin {
  static define(_: unknown, __?: unknown) { return {}; }
}
export const StateEffect = {
  define: <T>() => ({
    is: (_: unknown): _ is { is: unknown; value: T } => false,
    of: (_: T) => ({}),
  }),
};
export const StateField = {
  define: <T>(_: unknown) => ({} as T),
};
export class RangeSetBuilder<T> {
  add(_from: number, _to: number, _value: T): void {}
  finish() { return {}; }
}

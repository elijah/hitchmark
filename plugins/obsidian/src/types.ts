/** Shared type definitions for the Hookmarks Obsidian plugin. */

export interface HookmarksSettings {
  /** Whether to show purple numbers in the editor margin */
  showPurpleNumbers: boolean;
  /** CSS color for purple number annotations */
  purpleNumberColor: string;
  /** URL of the local hk HTTP server (when using hk serve) */
  daemonUrl: string;
  /** Copy URI to clipboard when clicking a purple number */
  copyOnClick: boolean;
  /** Absolute path to the hk binary (empty = auto-detect) */
  hkBinaryPath: string;
}

export const DEFAULT_SETTINGS: HookmarksSettings = {
  showPurpleNumbers: true,
  purpleNumberColor: "#888",
  daemonUrl: "http://localhost:7878",
  copyOnClick: true,
  hkBinaryPath: "",
};

export interface ParagraphInfo {
  /** Trimmed paragraph text */
  text: string;
  /** Computed purple ID (6-char base58) */
  id: string;
  /** Start character offset in the document */
  from: number;
  /** End character offset in the document */
  to: number;
}

export interface LinkRecord {
  source: string;
  target: string;
  note?: string;
  created_at?: string;
}

export interface HKResult<T> {
  ok: true;
  value: T;
}

export interface HKError {
  ok: false;
  error: string;
}

export type HKResponse<T> = HKResult<T> | HKError;

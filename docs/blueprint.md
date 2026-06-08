Invoked skill: blueprint

Ran terminal command

I'll research the Hookmarks protocol and purple numbers concepts, then produce the full blueprint plan.

Invoked skill: deep-research

I don't have firecrawl/exa MCP tools configured, so I'll draw on existing knowledge of Hookmarks and purple numbers — both are well-documented — and proceed directly to the blueprint.

---

## hitchmark — Project Blueprint

### What we're building

| Capability | Description |
|---|---|
| `hook://` URI scheme | Stable, addressable links to files, web pages, emails, and intra-document locations |
| Purple Numbers | Stable paragraph-level IDs rendered as small margin numbers, linkable via URI fragment |
| Core library | Cross-platform Rust crate: URI parsing, link graph storage, purple-number assignment |
| CLI | Cross-platform terminal tool for scripting and shell integration |
| macOS app | SwiftUI menu-bar app, URL-scheme handler, AppleScript bridges to common apps |
| Linux daemon | Rust daemon, DBus, xdg-open, XDG URI handler registration |
| Obsidian plugin | TypeScript community plugin: purple numbers in editor, bidirectional link sidebar, hook:// navigation |

---

### Monorepo Layout (target)

```
hitchmark/
├── specs/
│   ├── uri-scheme.md          # normative spec for hook:// URIs
│   └── purple-numbers.md      # normative spec for paragraph addressing
├── crates/
│   ├── hitchmark-core/        # URI, storage, purple-number engine
│   ├── hitchmark-cli/         # CLI binary (wraps core)
│   └── hitchmark-daemon/      # Linux system daemon
├── apps/
│   ├── macos/                 # Swift package / Xcode project
│   └── linux-tray/            # Rust (iced or tauri) system tray
├── plugins/
│   └── obsidian/              # TypeScript Obsidian community plugin
├── docs/                      # mdBook site
├── .github/workflows/
└── Cargo.toml                 # workspace root
```

---

### Step-by-step Construction Plan

#### **Step 0 — Repository Governance & Monorepo Scaffold**
*Serial prerequisite. All other steps depend on this.*

**Context brief:** Empty git repo at `~/hitchmark`. No code yet.

Tasks:
- Choose license (MIT recommended for ecosystem friendliness)
- Write `README.md` with mission, architecture overview, and component map
- Write `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`
- Initialize Cargo workspace (`Cargo.toml`) with members pointing at all future crates
- Add `.github/workflows/ci.yml`: `cargo test`, `cargo clippy`, `cargo fmt --check` on push to `main`
- Add `.gitignore`, `rust-toolchain.toml`, `rustfmt.toml`
- Create empty stub directories for all components with `.gitkeep`
- Commit: `chore: initial monorepo scaffold`

Exit criteria: `cargo check` passes on workspace; CI green on a draft PR.

---

#### **Step 1 — Normative Specifications** *(can run parallel with Step 0's tail)*
*Serial before Steps 2–6 since all components must implement the same spec.*

**Context brief:** We need canonical, versioned specs so macOS, Linux, CLI, and Obsidian interop perfectly.

Tasks — `specs/uri-scheme.md`:
- Define URI grammar:
  ```
  hook://file/<base64url-encoded-absolute-path>[#para-<purple-id>]
  hook://bookmark/<uuid>          (for non-file resources: URLs, emails)
  hook://x-callback-url/<action>  (for app-to-app callbacks, compat with Hook)
  ```
- Define content-identity rules (path vs. content hash for file stability)
- Define link metadata schema (source URI, target URI, created-at, tags, note)
- Version the spec as `v0.1`

Tasks — `specs/purple-numbers.md`:
- Define what constitutes a "paragraph" across file types (Markdown, plain text, PDF, code)
- Define purple-ID generation: content hash of paragraph text → base-58 short code (6 chars), collision-resistant within document
- Define stability contract: IDs survive paragraph reordering; invalidated only if content changes beyond a threshold (Levenshtein distance > 50%)
- Define rendering spec: right-margin, superscript `§abc123`, CSS variables for theming
- Define URI fragment format: `#para-<purple-id>`

Exit criteria: Both specs reviewed by at least one other person; merged to `main`.

---

#### **Step 2 — `hitchmark-core` Rust Crate**
*Depends on Step 1. This is the heart of everything.*

**Context brief:** Cross-platform Rust library. No I/O dependencies beyond `rusqlite`. Must compile on macOS and Linux. Other components link against this.

Tasks:
- `Cargo.toml`: deps = `rusqlite`, `serde`, `serde_json`, `base64`, `sha2`, `uuid`, `thiserror`, `url`
- `src/uri.rs`: Parse/serialize `hook://` URIs; validate per spec
- `src/purple.rs`: Assign stable IDs to paragraphs; compute similarity for stability; Markdown paragraph splitter
- `src/store.rs`: SQLite-backed `LinkStore` — CRUD for links, bookmarks, purple-number mappings
- `src/schema.sql`: Migrations embedded via `include_str!`
- `src/lib.rs`: Public API surface — `LinkStore::open(path)`, `create_link()`, `list_links_for()`, `resolve_uri()`
- Unit tests for URI round-trips, purple-ID stability, link CRUD
- Feature flag `wasm` for future browser/Obsidian WASM build

Exit criteria: `cargo test` passes; `cargo clippy -- -D warnings` clean.

---

#### **Step 3 — `hitchmark-cli` Binary**
*Depends on Step 2. Can be developed alongside Step 4.*

**Context brief:** Single `hk` binary. Links documents, queries the graph, opens hook:// URIs. Works headlessly in scripts.

Tasks:
- `Cargo.toml`: deps = `clap` (derive), `hitchmark-core`, `opener` (cross-platform open), `dirs`
- Subcommands:
  - `hk link <uri-a> <uri-b> [--note "..."]` — create bidirectional link
  - `hk unlink <uri-a> <uri-b>`
  - `hk list <uri>` — show all links for a resource
  - `hk open <hook-uri>` — resolve and open
  - `hk file <path>` — print the `hook://` URI for a file
  - `hk purple <file> [--format markdown]` — annotate a file with purple numbers
- Shell completions: `hk completions bash|zsh|fish`
- Config file: `~/.config/hookmarks/config.toml`
- Man page via `clap_mangen`

Exit criteria: `hk link`, `hk list`, `hk open` work end-to-end on Linux and macOS in CI.

---

#### **Step 4 — macOS App (Swift)**
*Depends on Step 2 (or its stable API). Independent of Steps 3, 5.*

**Context brief:** SwiftUI menu-bar app. Registers `hook://` URL scheme system-wide. Bridges to other macOS apps via AppleScript/accessibility. Users install this once; it runs in background.

Tasks:
- New Xcode project at `apps/macos/` (Swift Package + app target)
- `Info.plist`: register `hook` URL scheme (`CFBundleURLSchemes`)
- `AppDelegate`: handle `application(_:open:)` → dispatch to Rust core via FFI or subprocess
- Rust FFI bridge: expose `hitchmark-core` as a C-compatible dylib (`cbindgen` headers); OR call `hk` subprocess (simpler, ship later)
- Menu-bar UI (SwiftUI `MenuBarExtra`):
  - "Copy hook:// link for current document" — uses accessibility/AppleScript to get frontmost app's current file
  - "Links to this document" — shows linked items, click to open
  - "Preferences" — storage path, hotkeys
- AppleScript bridges for: Finder (selected file), Safari/Chrome (URL), Mail (message), Notes, BBEdit, Xcode
- Global hotkey (⌃⌥H) to trigger link creation
- Sparkle for auto-update
- Notarization + code signing setup notes in `apps/macos/SIGNING.md`

Exit criteria: App launches; `hook://` URI pasted into Safari opens and resolves; "Copy link" works for a Finder file.

---

#### **Step 5 — Linux Daemon (`hitchmark-daemon`)**
*Depends on Step 2. Independent of Steps 3, 4.*

**Context brief:** Rust background service. Handles `hook://` URIs via xdg-open. Provides DBus interface for desktop shell integration.

Tasks:
- `Cargo.toml`: deps = `hitchmark-core`, `zbus` (async DBus), `tokio`, `notify` (file watching), `xdg`
- DBus service name: `org.hitchmark.Daemon`
- DBus interface: `OpenURI(uri: String)`, `CreateLink(a: String, b: String)`, `ListLinks(uri: String) -> Vec<String>`
- Register `hook` URI scheme via `.desktop` file:
  ```
  apps/linux-tray/hitchmark.desktop
  MimeType=x-scheme-handler/hook;
  ```
- `scripts/install-linux.sh`: runs `xdg-mime default hitchmark.desktop x-scheme-handler/hook`
- Optional: system tray via `ksni` or `tauri-plugin-system-tray` (feature-flagged, not required for v0.1)
- Systemd user service unit file

Exit criteria: On Ubuntu/Fedora CI: `xdg-open hook://file/...` resolves via daemon; DBus `CreateLink` call works.

---

#### **Step 6 — Obsidian Community Plugin**
*Depends on Step 1 (spec) and Step 2 (for WASM or HTTP API). Independent of Steps 3, 4, 5.*

**Context brief:** TypeScript plugin for Obsidian desktop. Adds purple numbers to editor, bidirectional link panel, and hook:// URI handling. Targets Obsidian API ≥ 1.4.

Tasks:
- Scaffold at `plugins/obsidian/` with `manifest.json`, `package.json`, `esbuild.config.mjs`
- `src/main.ts`: `Plugin` subclass, register `hook` URI protocol handler
- **Purple numbers feature**:
  - CodeMirror 6 `ViewPlugin` that reads paragraphs and renders `§<id>` in the right gutter
  - Purple IDs computed client-side (match Rust algorithm in TypeScript for consistency)
  - Settings: show/hide, copy-on-click, color theme
- **Link panel** (`src/link-panel.ts`): `ItemView` sidebar showing all hook:// links for the active note
- **Commands**:
  - "Copy hook:// link for active note" → clipboard
  - "Copy hook:// link for current paragraph" → clipboard with `#para-<id>` fragment
  - "Open linked documents" → opens link panel
- **Settings tab**: hookmarks daemon URL (for native bridge), purple number rendering options
- Native bridge: HTTP to local `hk serve` or DBus (via `window.__native_bridge` on desktop)
- CI: `npm run build`, lint with ESLint, tests with Jest

Exit criteria: Plugin loads in Obsidian 1.5+; purple numbers render in a test vault; "copy hook:// link" works; panel shows links.

---

#### **Step 7 — Documentation Site**
*Depends on Steps 1–6 being at least draft-complete.*

**Context brief:** mdBook site at `docs/`. Deployed to GitHub Pages.

Tasks:
- `docs/book.toml`, `docs/src/SUMMARY.md`
- Chapters: Introduction, Getting Started (macOS / Linux / CLI / Obsidian), URI Scheme Reference, Purple Numbers deep-dive, Extending (plugin API), Contributing
- GitHub Actions workflow: `mdbook build` → `gh-pages` branch on push to `main`
- Logo/wordmark (SVG, simple)

---

### Dependency Graph (summary)

```
Step 0 (scaffold)
  └─► Step 1 (specs)
        └─► Step 2 (core crate)
              ├─► Step 3 (CLI)        ─┐
              ├─► Step 4 (macOS app)  ─┤─► Step 7 (docs)
              ├─► Step 5 (Linux)      ─┘
              └─► Step 6 (Obsidian — also needs Step 1)
```

**Parallel opportunities:** Steps 3, 4, 5, and 6 can all be developed concurrently once Steps 1 and 2 are stable. Assign to separate worktrees/agents.

---

### Key Technical Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Core language | Rust | Single binary, cross-platform, memory safe, FFI to Swift |
| macOS UI | SwiftUI + `MenuBarExtra` | Native feel, macOS 13+, small footprint |
| macOS↔Core bridge | Subprocess (`hk`) initially, FFI later | Ships faster; FFI is a perf optimization |
| Linux IPC | zbus (DBus) | Standard Linux desktop protocol |
| Storage | SQLite (rusqlite) | Zero-server, portable, embeds in every component |
| Purple ID algorithm | SHA-256(paragraph) → base-58 → first 6 chars | Stable across reorder, human-readable |
| Obsidian bridge | Local HTTP server (`hk serve`) | Works without native code in Obsidian sandbox |
| License | MIT | Maximum ecosystem compatibility, plugin store friendly |

---

### Risks & Mitigations

| Risk | Mitigation |
|---|---|
| Purple ID collisions within a document | Salt with doc path; detect at write time and extend to 8 chars |
| macOS notarization friction | Document steps early; use GitHub Actions with secrets for signing |
| Obsidian plugin review timeline (2–8 weeks) | Publish as BRAT-installable from day one |
| hook:// conflicts with real Hookmarks app | Add `x-hitchmark://` alias; document coexistence |
| FFI complexity macOS↔Rust | Ship subprocess bridge first; FFI is a v0.2 feature |

---

### Suggested Execution Order for a Solo Developer

1. **Week 1:** Step 0 + Step 1 (scaffold + specs) — most leverage per hour
2. **Week 2–3:** Step 2 (core crate) — foundation for everything
3. **Week 4:** Step 3 (CLI) — immediately useful, validates core
4. **Week 5–6:** Step 6 (Obsidian plugin) — highest visible impact for likely users
5. **Week 7–8:** Step 4 (macOS app) — platform showcase
6. **Week 9:** Step 5 (Linux daemon) — completeness
7. **Week 10:** Step 7 (docs) — community launch

---

Shall I start executing **Step 0** (scaffold) now, or would you like to adjust any architectural decisions first — particularly around the macOS↔Core bridge approach, purple number algorithm, or Obsidian plugin scope?
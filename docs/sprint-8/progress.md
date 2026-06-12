# Sprint 8 Progress

## Scope

Implement v0.6.0 items 2, 3, 4, 5:

1. Linux packaging (`.deb` / `.rpm`) in release pipeline
2. Windows tray auto-start watch support
3. Obsidian plugin release polish
4. `hk serve` hardening (signal handling + pid file)

## Updates

- 2026-06-11: Sprint started on branch `feature/sprint-8`; blockers checked (`gh issue list` returned none).
- 2026-06-11: Implemented tray startup enhancement:
  - Added `auto_start_watch` to tray config/defaults
  - Tray now auto-starts `hk serve` and `hk watch` on launch when configured
  - Bridge now respects configured `hk_path` and validates `/open` HTTP status
- 2026-06-11: Implemented Linux release packaging:
  - Added `.deb` / `.rpm` metadata in `crates/hitchmark-cli/Cargo.toml`
  - Extended `release.yml` Linux job to build/upload `hk-linux-x86_64.deb` and `hk-linux-x86_64.rpm`
- 2026-06-11: Implemented Obsidian release polish:
  - Bumped plugin version to `0.5.0` in `manifest.json` and `package.json`
  - Added `plugins/obsidian/versions.json`
  - Added Obsidian bundle artifact (`hitchmark-obsidian.zip`) to release workflow
  - Updated Obsidian docs naming/paths (Hookmarks → Hitchmark)
- 2026-06-11: Implemented `hk serve` hardening:
  - Added `--pid-file` flag support
  - Added Unix SIGTERM shutdown handling (in addition to Ctrl-C)
  - Added PID file lifecycle guard (create on startup, remove on shutdown)

#!/usr/bin/env bash
# install-serve.sh — install/uninstall the Hitchmark auto-start agent on macOS
# Usage: ./install-serve.sh [--uninstall]
set -euo pipefail

LABEL="app.hitchmark.serve"
PLIST_SRC="$(cd "$(dirname "$0")" && pwd)/app.hitchmark.serve.plist"
PLIST_DST="$HOME/Library/LaunchAgents/$LABEL.plist"
HK_PATH="${HK_PATH:-/usr/local/bin/hk}"

# ── helper ────────────────────────────────────────────────────────────────────
die() { echo "error: $*" >&2; exit 1; }
info() { echo "==> $*"; }

if [[ "${1:-}" == "--uninstall" ]]; then
    launchctl unload "$PLIST_DST" 2>/dev/null || true
    rm -f "$PLIST_DST"
    info "Hitchmark auto-start removed."
    exit 0
fi

# ── install ───────────────────────────────────────────────────────────────────
[[ -f "$PLIST_SRC" ]] || die "plist not found: $PLIST_SRC"
[[ -x "$HK_PATH" ]] || {
    # Try Cargo bin as fallback
    HK_PATH="$HOME/.cargo/bin/hk"
    [[ -x "$HK_PATH" ]] || die "hk binary not found. Set HK_PATH or install hitchmark first."
}

# Write plist with the actual hk path substituted
mkdir -p "$HOME/Library/LaunchAgents"
sed "s|/usr/local/bin/hk|$HK_PATH|g" "$PLIST_SRC" > "$PLIST_DST"

# Unload stale copy if present, then load
launchctl unload "$PLIST_DST" 2>/dev/null || true
launchctl load -w "$PLIST_DST"

info "Hitchmark serve agent installed and started."
info "  plist : $PLIST_DST"
info "  logs  : /tmp/hitchmark-serve.log (stdout), /tmp/hitchmark-serve.err (stderr)"
info ""
info "To stop:    launchctl unload $PLIST_DST"
info "To remove:  $0 --uninstall"

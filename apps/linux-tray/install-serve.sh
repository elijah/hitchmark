#!/usr/bin/env bash
# install-serve.sh — install/uninstall the Hitchmark systemd --user service on Linux
# Installs: hitchmark-serve.service (hk serve HTTP bridge)
# Optional: hitchmark-daemon.service (background DBus daemon)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SYSTEMD_USER="$HOME/.config/systemd/user"

die()  { echo "error: $*" >&2; exit 1; }
info() { echo "==> $*"; }

UNINSTALL=false
for arg in "$@"; do
    [[ "$arg" == "--uninstall" ]] && UNINSTALL=true
done

if $UNINSTALL; then
    systemctl --user stop  hitchmark-serve.service  2>/dev/null || true
    systemctl --user disable hitchmark-serve.service 2>/dev/null || true
    rm -f "$SYSTEMD_USER/hitchmark-serve.service"
    systemctl --user daemon-reload
    info "Hitchmark serve service removed."
    exit 0
fi

# Locate hk binary
HK_PATH="${HK_PATH:-}"
for candidate in "$HOME/.cargo/bin/hk" "/usr/local/bin/hk" "/usr/bin/hk"; do
    if [[ -x "$candidate" ]]; then
        HK_PATH="$candidate"
        break
    fi
done
[[ -n "$HK_PATH" ]] || die "hk binary not found. Build and install hitchmark first, or set HK_PATH."

mkdir -p "$SYSTEMD_USER"

# Write service file with correct ExecStart path
sed "s|%h/.cargo/bin/hk|$HK_PATH|g" \
    "$SCRIPT_DIR/hitchmark-serve.service" \
    > "$SYSTEMD_USER/hitchmark-serve.service"

systemctl --user daemon-reload
systemctl --user enable --now hitchmark-serve.service

info "hitchmark-serve.service installed and started."
info "  logs: journalctl --user -u hitchmark-serve.service -f"
info ""
info "To stop:    systemctl --user stop hitchmark-serve"
info "To remove:  $0 --uninstall"

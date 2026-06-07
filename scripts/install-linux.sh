#!/usr/bin/env bash
# install-linux.sh — Install Hookmarks on Linux
#
# Usage:
#   ./scripts/install-linux.sh              # Install built release binary
#   ./scripts/install-linux.sh --uninstall  # Remove Hookmarks
#
# This script:
#   1. Installs the hookmarks-daemon binary to ~/.local/bin/
#   2. Registers the hook:// URI scheme via xdg-mime
#   3. Installs the systemd user service and enables it
#   4. Verifies the installation

set -euo pipefail

BINARY_NAME="hookmarks-daemon"
BINARY_SRC="./target/release/${BINARY_NAME}"
INSTALL_DIR="${HOME}/.local/bin"
DESKTOP_FILE="./apps/linux-tray/not-hookmarks.desktop"
DESKTOP_DIR="${HOME}/.local/share/applications"
SYSTEMD_DIR="${HOME}/.config/systemd/user"
SERVICE_FILE="./apps/linux-tray/hookmarks-daemon.service"
DATA_DIR="${HOME}/.local/share/hookmarks"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${BLUE}[hookmarks]${NC} $*"; }
ok()    { echo -e "${GREEN}[hookmarks]${NC} ✅ $*"; }
error() { echo -e "${RED}[hookmarks]${NC} ❌ $*" >&2; }

# ----- Uninstall -----
if [[ "${1:-}" == "--uninstall" ]]; then
    info "Uninstalling Hookmarks..."

    systemctl --user stop hookmarks-daemon 2>/dev/null || true
    systemctl --user disable hookmarks-daemon 2>/dev/null || true
    rm -f "${SYSTEMD_DIR}/hookmarks-daemon.service"
    systemctl --user daemon-reload 2>/dev/null || true

    xdg-mime default "" x-scheme-handler/hook 2>/dev/null || true
    rm -f "${DESKTOP_DIR}/not-hookmarks.desktop"
    update-desktop-database "${DESKTOP_DIR}" 2>/dev/null || true

    rm -f "${INSTALL_DIR}/${BINARY_NAME}"

    ok "Hookmarks uninstalled. Data preserved at ${DATA_DIR}"
    exit 0
fi

# ----- Pre-flight checks -----
info "Checking prerequisites..."

if [[ ! -f "${BINARY_SRC}" ]]; then
    error "Binary not found at ${BINARY_SRC}"
    echo ""
    echo "Build the release binary first:"
    echo "  cargo build --release -p hookmarks-daemon"
    exit 1
fi

if ! command -v xdg-mime &>/dev/null; then
    error "xdg-mime not found. Install xdg-utils:"
    echo "  sudo apt install xdg-utils    # Debian/Ubuntu"
    echo "  sudo dnf install xdg-utils    # Fedora"
    exit 1
fi

# ----- Install binary -----
info "Installing binary to ${INSTALL_DIR}..."
mkdir -p "${INSTALL_DIR}"
install -m 755 "${BINARY_SRC}" "${INSTALL_DIR}/${BINARY_NAME}"
ok "Binary installed: ${INSTALL_DIR}/${BINARY_NAME}"

# Ensure ~/.local/bin is in PATH
if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
    echo ""
    echo "⚠️  ${INSTALL_DIR} is not in your PATH."
    echo "    Add this to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo "    export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    echo ""
fi

# ----- Install desktop file and register URI scheme -----
info "Installing .desktop file and registering hook:// URI scheme..."
mkdir -p "${DESKTOP_DIR}"
install -m 644 "${DESKTOP_FILE}" "${DESKTOP_DIR}/not-hookmarks.desktop"
update-desktop-database "${DESKTOP_DIR}" 2>/dev/null || true
xdg-mime default not-hookmarks.desktop x-scheme-handler/hook
ok "hook:// URI scheme registered"

# Verify registration
HANDLER=$(xdg-mime query default x-scheme-handler/hook 2>/dev/null || echo "")
if [[ "${HANDLER}" == "not-hookmarks.desktop" ]]; then
    ok "URI scheme verified: hook:// → ${HANDLER}"
else
    error "URI scheme registration may have failed (got: ${HANDLER})"
fi

# ----- Install systemd user service -----
if command -v systemctl &>/dev/null; then
    info "Installing systemd user service..."
    mkdir -p "${SYSTEMD_DIR}"

    # Substitute %h with actual home directory in service file
    sed "s|%h|${HOME}|g" "${SERVICE_FILE}" > "${SYSTEMD_DIR}/hookmarks-daemon.service"

    systemctl --user daemon-reload
    systemctl --user enable hookmarks-daemon
    systemctl --user start hookmarks-daemon

    # Give it a moment to start
    sleep 1

    if systemctl --user is-active --quiet hookmarks-daemon; then
        ok "Daemon running (systemd user service active)"
    else
        error "Daemon failed to start. Check logs:"
        echo "  journalctl --user -u hookmarks-daemon -n 20"
    fi
else
    info "systemctl not available — skipping service installation"
    echo "  Start daemon manually: ${INSTALL_DIR}/${BINARY_NAME}"
fi

# ----- Create data directory -----
mkdir -p "${DATA_DIR}"
ok "Data directory: ${DATA_DIR}"

# ----- Summary -----
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ok "Hookmarks installed successfully!"
echo ""
echo "  Test it:"
echo "    xdg-open hook://file/$(echo -n /home/you/document.md | base64 -w0)"
echo ""
echo "  DBus interface (once daemon is running):"
echo "    gdbus call --session \\"
echo "      --dest org.not_hookmarks.Daemon \\"
echo "      --object-path /org/not_hookmarks/Daemon \\"
echo "      --method org.not_hookmarks.Daemon1.FileToUri \\"
echo "      '/home/you/document.md'"
echo ""
echo "  Uninstall:"
echo "    ./scripts/install-linux.sh --uninstall"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

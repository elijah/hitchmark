#!/usr/bin/env bash
# install.sh — Install Hookmarks XDG Desktop Actions
#
# Installs right-click context menu actions for:
#   - GNOME Nautilus (Files)
#   - KDE Dolphin (Plasma 5 + 6)
#   - Xfce Thunar
#   - Cinnamon/Mint Nemo
#
# Actions installed:
#   1. Copy hook:// URI        — file → clipboard
#   2. Link Two Files          — 2 files → hk link
#   3. Show Links              — file → linked documents dialog
#   4. Open hook:// URI        — clipboard → hk open  (keyboard shortcut helper)
#
# Usage:
#   ./install.sh               # auto-detect file managers and install
#   ./install.sh --uninstall   # remove all installed files
#   ./install.sh --dry-run     # show what would be installed without doing it

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_SRC="${SCRIPT_DIR}/scripts"

# Installation targets
HK_SCRIPTS_DEST="${HOME}/.local/share/hookmarks/scripts"
BIN_DIR="${HOME}/.local/bin"

# DE-specific paths
NAUTILUS_SCRIPTS_DIR="${HOME}/.local/share/nautilus/scripts"
KDE5_SERVICEMENU_DIR="${HOME}/.local/share/kservices5/ServiceMenus"
KDE6_SERVICEMENU_DIR="${HOME}/.local/share/kio/servicemenus"
NEMO_ACTIONS_DIR="${HOME}/.local/share/nemo/actions"
THUNAR_UCA="${HOME}/.config/Thunar/uca.xml"

DRY_RUN=false
UNINSTALL=false

RED='\033[0;31m'; GREEN='\033[0;32m'; BLUE='\033[0;34m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()    { echo -e "${BLUE}[hookmarks]${NC} $*"; }
ok()      { echo -e "${GREEN}[hookmarks]${NC} ✅  $*"; }
warn()    { echo -e "${YELLOW}[hookmarks]${NC} ⚠️  $*"; }
error()   { echo -e "${RED}[hookmarks]${NC} ❌  $*" >&2; }
dry_run() { echo -e "${YELLOW}[dry-run]${NC}  $*"; }

install_file() {
    local src="$1" dest="$2" mode="${3:-644}"
    local dest_dir
    dest_dir="$(dirname "$dest")"
    if $DRY_RUN; then
        dry_run "cp \"$src\" \"$dest\"  (mode $mode)"
        return
    fi
    mkdir -p "$dest_dir"
    cp "$src" "$dest"
    chmod "$mode" "$dest"
}

symlink_file() {
    local src="$1" dest="$2"
    if $DRY_RUN; then
        dry_run "ln -sf \"$src\" \"$dest\""
        return
    fi
    mkdir -p "$(dirname "$dest")"
    ln -sf "$src" "$dest"
}

remove_file() {
    local path="$1"
    if $DRY_RUN; then
        dry_run "rm -f \"$path\""
        return
    fi
    rm -f "$path"
}

# ────────────────────────────────────────────────────────────────
# Arg parsing
# ────────────────────────────────────────────────────────────────
for arg in "$@"; do
    case "$arg" in
        --uninstall) UNINSTALL=true ;;
        --dry-run)   DRY_RUN=true ;;
        --help|-h)
            echo "Usage: $(basename "$0") [--uninstall] [--dry-run]"
            exit 0 ;;
    esac
done

# ────────────────────────────────────────────────────────────────
# Uninstall
# ────────────────────────────────────────────────────────────────
if $UNINSTALL; then
    info "Uninstalling Hookmarks XDG actions..."

    remove_file "${NAUTILUS_SCRIPTS_DIR}/Hookmarks - Copy URI"
    remove_file "${NAUTILUS_SCRIPTS_DIR}/Hookmarks - Link Files"
    remove_file "${NAUTILUS_SCRIPTS_DIR}/Hookmarks - Show Links"

    remove_file "${KDE5_SERVICEMENU_DIR}/hookmarks.desktop"
    remove_file "${KDE6_SERVICEMENU_DIR}/hookmarks.desktop"

    remove_file "${NEMO_ACTIONS_DIR}/hookmarks-copy-uri.nemo_action"
    remove_file "${NEMO_ACTIONS_DIR}/hookmarks-link-files.nemo_action"
    remove_file "${NEMO_ACTIONS_DIR}/hookmarks-show-links.nemo_action"

    remove_file "${BIN_DIR}/hookmarks-copy-uri"
    remove_file "${BIN_DIR}/hookmarks-link-files"
    remove_file "${BIN_DIR}/hookmarks-show-links"
    remove_file "${BIN_DIR}/hookmarks-open-uri"

    # Note: we don't touch Thunar uca.xml on uninstall — it's a merge, not ownership
    warn "Thunar UCA entries must be removed manually from ${THUNAR_UCA}"
    warn "Core scripts left at ${HK_SCRIPTS_DEST} — remove manually if desired"

    ok "Hookmarks XDG actions uninstalled"
    exit 0
fi

# ────────────────────────────────────────────────────────────────
# Detect desktop environment
# ────────────────────────────────────────────────────────────────
DE="${XDG_CURRENT_DESKTOP:-unknown}"
info "Desktop environment: ${DE:-unknown}"

# ────────────────────────────────────────────────────────────────
# Step 1: Install core scripts to ~/.local/share/hookmarks/scripts/
# ────────────────────────────────────────────────────────────────
info "Installing core scripts to ${HK_SCRIPTS_DEST}/"

for script in hk-common.sh hk-copy-uri.sh hk-link-files.sh hk-show-links.sh hk-open-uri.sh; do
    install_file "${SCRIPTS_SRC}/${script}" "${HK_SCRIPTS_DEST}/${script}" 755
done

ok "Core scripts installed"

# ────────────────────────────────────────────────────────────────
# Step 2: Install wrapper symlinks in ~/.local/bin/
#   Named without .sh so file managers can call them as commands
# ────────────────────────────────────────────────────────────────
info "Installing CLI wrappers in ${BIN_DIR}/"

for cmd in copy-uri link-files show-links open-uri; do
    symlink_file "${HK_SCRIPTS_DEST}/hk-${cmd}.sh" "${BIN_DIR}/hookmarks-${cmd}"
done

ok "CLI wrappers installed (hookmarks-copy-uri, hookmarks-link-files, etc.)"

# Ensure ~/.local/bin is on PATH
if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
    warn "${BIN_DIR} is not in your PATH."
    warn "Add this to ~/.bashrc or ~/.zshrc:"
    warn "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
fi

# ────────────────────────────────────────────────────────────────
# Step 3: Nautilus (GNOME Files)
# ────────────────────────────────────────────────────────────────
INSTALLED_NAUTILUS=false
if command -v nautilus &>/dev/null || [[ -d "${NAUTILUS_SCRIPTS_DIR}" ]]; then
    info "Installing Nautilus scripts..."
    for script in "Hookmarks - Copy URI" "Hookmarks - Link Files" "Hookmarks - Show Links"; do
        install_file "${SCRIPT_DIR}/nautilus/${script}" \
                     "${NAUTILUS_SCRIPTS_DIR}/${script}" 755
    done
    INSTALLED_NAUTILUS=true
    ok "Nautilus scripts installed → right-click → Scripts → Hookmarks"
else
    info "Nautilus not detected — skipping"
fi

# ────────────────────────────────────────────────────────────────
# Step 4: KDE Dolphin ServiceMenu
# ────────────────────────────────────────────────────────────────
INSTALLED_KDE=false
if command -v dolphin &>/dev/null || command -v kservice5 &>/dev/null \
    || [[ "${DE}" == *"KDE"* ]] || [[ "${DE}" == *"kde"* ]]; then
    info "Installing KDE ServiceMenu..."
    # Install for both Plasma 5 and 6
    install_file "${SCRIPT_DIR}/kde/hookmarks.desktop" \
                 "${KDE5_SERVICEMENU_DIR}/hookmarks.desktop"
    install_file "${SCRIPT_DIR}/kde/hookmarks.desktop" \
                 "${KDE6_SERVICEMENU_DIR}/hookmarks.desktop"
    # Reload KDE service menus if tools available
    if ! $DRY_RUN; then
        kbuildsycoca5 --noincremental 2>/dev/null || true
        kbuildsycoca6 --noincremental 2>/dev/null || true
    fi
    INSTALLED_KDE=true
    ok "KDE ServiceMenu installed → right-click → Hookmarks"
else
    info "KDE Dolphin not detected — skipping"
fi

# ────────────────────────────────────────────────────────────────
# Step 5: Nemo (Cinnamon / Mint)
# ────────────────────────────────────────────────────────────────
INSTALLED_NEMO=false
if command -v nemo &>/dev/null || [[ "${DE}" == *"Cinnamon"* ]] \
    || [[ "${DE}" == *"cinnamon"* ]]; then
    info "Installing Nemo actions..."
    for action in hookmarks-copy-uri hookmarks-link-files hookmarks-show-links; do
        install_file "${SCRIPT_DIR}/nemo/${action}.nemo_action" \
                     "${NEMO_ACTIONS_DIR}/${action}.nemo_action"
    done
    INSTALLED_NEMO=true
    ok "Nemo actions installed → right-click → Hookmarks"
else
    info "Nemo not detected — skipping"
fi

# ────────────────────────────────────────────────────────────────
# Step 6: Thunar (Xfce)  — merge into uca.xml
# ────────────────────────────────────────────────────────────────
INSTALLED_THUNAR=false
if command -v thunar &>/dev/null || [[ "${DE}" == *"XFCE"* ]] \
    || [[ "${DE}" == *"xfce"* ]]; then
    info "Merging Thunar custom actions..."

    UCA_SRC="${SCRIPT_DIR}/thunar/uca-hookmarks.xml"

    if $DRY_RUN; then
        dry_run "Merge ${UCA_SRC} into ${THUNAR_UCA}"
    else
        if [[ ! -f "$THUNAR_UCA" ]]; then
            mkdir -p "$(dirname "$THUNAR_UCA")"
            printf '<?xml version="1.0" encoding="UTF-8"?>\n<actions>\n</actions>\n' \
                > "$THUNAR_UCA"
        fi

        # Check if already installed
        if grep -q "hookmarks-copy-uri-001" "$THUNAR_UCA" 2>/dev/null; then
            warn "Thunar UCA entries already present — skipping merge"
        else
            # Merge: insert before closing </actions>
            python3 - "$THUNAR_UCA" "$UCA_SRC" <<'PYEOF'
import sys, re

uca_path = sys.argv[1]
new_actions_path = sys.argv[2]

with open(uca_path, 'r') as f:
    content = f.read()
with open(new_actions_path, 'r') as f:
    new_xml = f.read()

# Extract only <action>...</action> blocks from the new_actions file
actions = re.findall(r'<action>.*?</action>', new_xml, re.DOTALL)
insert = '\n' + '\n\n'.join(actions) + '\n'

# Insert before </actions>
updated = re.sub(r'(</actions>)', insert + r'\1', content, count=1)

with open(uca_path, 'w') as f:
    f.write(updated)
print("Merged.")
PYEOF
            INSTALLED_THUNAR=true
            ok "Thunar custom actions merged → right-click → Hookmarks"
        fi
    fi
else
    info "Thunar not detected — skipping"
fi

# ────────────────────────────────────────────────────────────────
# Summary
# ────────────────────────────────────────────────────────────────
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ok "Hookmarks XDG actions installed!"
echo ""
echo "  Installed for:"
$INSTALLED_NAUTILUS && echo "    ✅  Nautilus (GNOME Files)"
$INSTALLED_KDE      && echo "    ✅  KDE Dolphin (Plasma 5 + 6)"
$INSTALLED_NEMO     && echo "    ✅  Nemo (Cinnamon / Linux Mint)"
$INSTALLED_THUNAR   && echo "    ✅  Thunar (Xfce)"

echo ""
echo "  Restart your file manager to pick up the changes."
echo "  Then right-click any file → Hookmarks/Scripts"
echo ""
echo "  Actions available:"
echo "    • Copy hook:// URI"
echo "    • Link Two Files (select 2 files)"
echo "    • Show Links"
echo "    • hookmarks-open-uri  (run from terminal / keyboard shortcut)"
echo ""
echo "  Optional: bind hookmarks-open-uri to a keyboard shortcut in"
echo "  your DE settings to open hook:// URIs from clipboard anywhere."
echo ""
if ! command -v hk &>/dev/null; then
    warn "hk not found in PATH. Install Hookmarks CLI:"
    echo "    cargo install hitchmark-cli"
    echo "    # or: brew install hookmarks"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

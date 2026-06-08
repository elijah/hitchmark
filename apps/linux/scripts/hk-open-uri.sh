#!/usr/bin/env bash
# hk-open-uri.sh — Open a hook:// URI from clipboard or first argument.
#
# Usage:
#   hk-open-uri.sh                    # reads hook:// URI from clipboard
#   hk-open-uri.sh "hook://file/..."  # open directly
#
# Intended for: keyboard shortcut, Rofi/dmenu launcher, or text-selection service.
# The macOS equivalent is "Open hook:// URI" (text selection service).

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=hk-common.sh
source "${SCRIPT_DIR}/hk-common.sh"

# Get URI: arg > clipboard > prompt
URI="${1:-}"

if [[ -z "$URI" ]]; then
    # Read from clipboard
    if command -v wl-paste &>/dev/null; then
        URI=$(wl-paste 2>/dev/null || echo "")
    elif command -v xclip &>/dev/null; then
        URI=$(xclip -selection clipboard -o 2>/dev/null || echo "")
    elif command -v xsel &>/dev/null; then
        URI=$(xsel --clipboard --output 2>/dev/null || echo "")
    fi
fi

# Extract first hook:// URI from potentially longer text
URI=$(echo "$URI" | grep -oP 'hook://[^\s"<>]+' | head -1 || echo "")

if [[ -z "$URI" ]]; then
    # Offer an input dialog if available
    if command -v zenity &>/dev/null; then
        URI=$(zenity --entry \
            --title="Hookmarks — Open URI" \
            --text="Paste a hook:// URI to open:" \
            --entry-text="hook://" 2>/dev/null || echo "")
    elif command -v kdialog &>/dev/null; then
        URI=$(kdialog --title "Hookmarks — Open URI" \
            --inputbox "Paste a hook:// URI to open:" "hook://" 2>/dev/null || echo "")
    fi
fi

if [[ -z "$URI" || "$URI" == "hook://" ]]; then
    notify_ui "dialog-error" "Hookmarks" \
        "No hook:// URI found in clipboard or selection."
    exit 1
fi

HK=$(locate_hk)
if [[ -z "$HK" ]]; then
    notify_ui "dialog-error" "Hookmarks" \
        "hk not found. Install with: brew install hookmarks"
    exit 1
fi

"$HK" open "$URI" 2>/dev/null || \
    notify_ui "dialog-error" "Hookmarks — Open Failed" \
        "Could not open: $URI"

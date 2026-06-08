#!/usr/bin/env bash
# hk-copy-uri.sh — Convert a file to a hook:// URI and copy to clipboard.
#
# Usage: hk-copy-uri.sh <file> [<file2> ...]
# Environment: HK_SERVER=http://127.0.0.1:2701 (optional override)
#
# Called by: Nautilus scripts, KDE ServiceMenu, Thunar UCA, Nemo actions

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=hk-common.sh
source "${SCRIPT_DIR}/hk-common.sh"

FILE="${1:-}"
if [[ -z "$FILE" ]]; then
    notify_ui "dialog-error" "Hookmarks" "No file provided."
    exit 1
fi

URI=$(hk_uri "$FILE") || exit 1
if [[ -z "$URI" ]]; then
    notify_ui "dialog-error" "Hookmarks" \
        "Could not get hook:// URI for: $(basename "$FILE")"
    exit 1
fi

copy_to_clipboard "$URI"
notify_ui "edit-copy" "Hookmarks" "Copied: $URI"

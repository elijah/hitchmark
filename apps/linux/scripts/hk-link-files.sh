#!/usr/bin/env bash
# hk-link-files.sh — Create a bidirectional Hookmarks link between two files.
#
# Usage: hk-link-files.sh <file1> <file2> [--note "optional note"]
# Environment: HK_SERVER=http://127.0.0.1:2701 (optional override)
#
# Called by: Nautilus scripts, KDE ServiceMenu, Thunar UCA, Nemo actions

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=hk-common.sh
source "${SCRIPT_DIR}/hk-common.sh"

FILE_A="${1:-}"
FILE_B="${2:-}"
NOTE="${3:-}"

if [[ -z "$FILE_A" || -z "$FILE_B" ]]; then
    notify_ui "dialog-error" "Hookmarks — Link Files" \
        "Select exactly two files to link.\n\nUsage: hk-link-files.sh <file1> <file2>" \
        --dialog
    exit 1
fi

URI_A=$(hk_uri "$FILE_A") || exit 1
URI_B=$(hk_uri "$FILE_B") || exit 1

if [[ -z "$URI_A" ]]; then
    notify_ui "dialog-error" "Hookmarks" "Could not resolve URI for: $(basename "$FILE_A")"
    exit 1
fi
if [[ -z "$URI_B" ]]; then
    notify_ui "dialog-error" "Hookmarks" "Could not resolve URI for: $(basename "$FILE_B")"
    exit 1
fi

hk_link "$URI_A" "$URI_B" "$NOTE"
notify_ui "emblem-symbolic-link" "Hookmarks — Files Linked ✓" \
    "$(basename "$FILE_A")  ↔  $(basename "$FILE_B")"

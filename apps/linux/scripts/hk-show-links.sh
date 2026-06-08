#!/usr/bin/env bash
# hk-show-links.sh — Show all Hookmarks links for a file in a dialog.
#
# Usage: hk-show-links.sh <file>
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
    notify_ui "dialog-error" "Hookmarks" "Could not resolve URI for: $(basename "$FILE")"
    exit 1
fi

JSON=$(hk_list "$URI")
FILENAME="$(basename "$FILE")"

# Parse JSON with python3 (available on all modern Linux distros)
FORMATTED=$(python3 - "$URI" <<'EOF'
import sys, json, os, base64

uri = sys.argv[1]

raw = sys.stdin.read().strip()
if not raw:
    print("No links found for this file.")
    sys.exit(0)

try:
    links = json.loads(raw)
except json.JSONDecodeError:
    print("(Could not parse response)")
    sys.exit(0)

if not links:
    print("No links found for this file.")
    sys.exit(0)

lines = []
for i, link in enumerate(links, 1):
    other = link["uri_b"] if link.get("uri_a") == uri else link.get("uri_a", "")
    note  = link.get("note") or ""
    # Decode base64url path for readability
    if other.startswith("hook://file/"):
        encoded = other[len("hook://file/"):].split("#")[0]
        encoded = encoded.replace("-", "+").replace("_", "/")
        pad = (4 - len(encoded) % 4) % 4
        try:
            path = base64.b64decode(encoded + "=" * pad).decode("utf-8", errors="replace")
            home = os.path.expanduser("~")
            if path.startswith(home):
                path = "~" + path[len(home):]
            other = path
        except Exception:
            pass
    note_str = f"  — {note}" if note else ""
    lines.append(f"{i}. {other}{note_str}")

print("\n".join(lines))
EOF
<<< "$JSON")

MSG="Links for $FILENAME:\n\n$FORMATTED"

# If no dialog available, copy the first URI as a fallback
if ! command -v zenity &>/dev/null && ! command -v kdialog &>/dev/null; then
    FIRST_URI=$(python3 -c "
import sys, json
d = json.loads(sys.stdin.read() or '[]')
uri = sys.argv[1]
links = d if isinstance(d, list) else []
if links:
    l = links[0]
    print(l['uri_b'] if l.get('uri_a') == uri else l.get('uri_a', ''))
" "$URI" <<< "$JSON")
    [[ -n "$FIRST_URI" ]] && copy_to_clipboard "$FIRST_URI"
    notify_ui "edit-find" "Hookmarks — Show Links" \
        "${FORMATTED:0:200}"
    exit 0
fi

notify_ui "edit-find" "Hookmarks — Links for $FILENAME" "$MSG" --dialog

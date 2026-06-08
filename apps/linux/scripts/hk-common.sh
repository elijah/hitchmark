#!/usr/bin/env bash
# hk-notify.sh — Portable UI notification helper.
# Source this file, then call: notify_ui ICON TITLE MESSAGE [--dialog]
#
# Icon names: edit-copy, edit-find, emblem-symbolic-link, dialog-error, dialog-information

notify_ui() {
    local icon="$1" title="$2" msg="$3" mode="${4:-}"

    # Full dialog (for multi-line content like link lists)
    if [[ "$mode" == "--dialog" ]]; then
        if command -v zenity &>/dev/null; then
            zenity --info --title="$title" --text="$msg" --width=400 2>/dev/null
            return
        elif command -v kdialog &>/dev/null; then
            kdialog --title "$title" --msgbox "$msg" 2>/dev/null
            return
        fi
        # Fallback: truncate to 200 chars for notify-send
        msg="${msg:0:200}"
    fi

    if command -v notify-send &>/dev/null; then
        notify-send -i "$icon" "$title" "$msg"
    elif command -v kdialog &>/dev/null; then
        kdialog --title "$title" --passivepopup "$msg" 4
    else
        echo "$title: $msg" >&2
    fi
}

# Copy text to the system clipboard (Wayland + X11)
copy_to_clipboard() {
    local text="$1"
    if command -v wl-copy &>/dev/null; then
        printf '%s' "$text" | wl-copy
    elif command -v xclip &>/dev/null; then
        printf '%s' "$text" | xclip -selection clipboard
    elif command -v xsel &>/dev/null; then
        printf '%s' "$text" | xsel --clipboard --input
    else
        notify_ui "dialog-error" "Hookmarks" \
            "No clipboard tool found. Install wl-clipboard (Wayland) or xclip (X11)."
        return 1
    fi
}

# Locate the hk binary
locate_hk() {
    if command -v hk &>/dev/null; then
        echo "hk"
        return
    fi
    for p in \
        "$HOME/.local/bin/hk" \
        "$HOME/.cargo/bin/hk" \
        "/usr/local/bin/hk" \
        "/opt/homebrew/bin/hk"
    do
        if [[ -x "$p" ]]; then
            echo "$p"
            return
        fi
    done
    echo ""
}

# Call hk serve HTTP API or fall back to hk CLI
hk_uri() {
    local path="$1"
    local hk
    hk=$(locate_hk)
    if [[ -z "$hk" ]]; then
        notify_ui "dialog-error" "Hookmarks" \
            "hk not found. Install with: brew install hookmarks"
        return 1
    fi

    # Try HTTP server first
    local server="${HK_SERVER:-http://127.0.0.1:2701}"
    local encoded
    encoded=$(python3 -c "import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1]))" "$path" 2>/dev/null \
        || python -c "import urllib,sys; print urllib.quote(sys.argv[1])" "$path" 2>/dev/null \
        || printf '%s' "$path" | sed 's| |%20|g')
    local uri
    uri=$(curl -sf --max-time 1 "${server}/uri?path=${encoded}" \
        | python3 -c "import sys,json; print(json.load(sys.stdin).get('uri',''))" 2>/dev/null)
    if [[ -n "$uri" ]]; then
        echo "$uri"
        return
    fi

    # Subprocess fallback
    "$hk" uri "$path" 2>/dev/null
}

hk_list() {
    local uri="$1"
    local hk
    hk=$(locate_hk)
    if [[ -z "$hk" ]]; then return 1; fi

    local server="${HK_SERVER:-http://127.0.0.1:2701}"
    local encoded
    encoded=$(python3 -c "import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1]))" "$uri" 2>/dev/null \
        || printf '%s' "$uri" | sed 's|:|%3A|g;s|/|%2F|g')
    local result
    result=$(curl -sf --max-time 2 "${server}/links?uri=${encoded}" 2>/dev/null)
    if [[ -n "$result" ]]; then
        echo "$result"
        return
    fi

    "$hk" list "$uri" --json 2>/dev/null
}

hk_link() {
    local uriA="$1" uriB="$2" note="${3:-}"
    local hk
    hk=$(locate_hk)
    if [[ -z "$hk" ]]; then return 1; fi

    local server="${HK_SERVER:-http://127.0.0.1:2701}"
    local body="{\"uri_a\":\"${uriA}\",\"uri_b\":\"${uriB}\"}"
    [[ -n "$note" ]] && body="{\"uri_a\":\"${uriA}\",\"uri_b\":\"${uriB}\",\"note\":\"${note}\"}"
    if curl -sf --max-time 2 -X POST \
            -H "Content-Type: application/json" \
            -d "$body" \
            "${server}/links" &>/dev/null; then
        return 0
    fi

    local args=("$uriA" "$uriB")
    [[ -n "$note" ]] && args+=("--note" "$note")
    "$hk" link "${args[@]}" 2>/dev/null
}

# Decode a hook://file/<base64url> URI back to a readable path
decode_hook_uri() {
    local uri="$1"
    local encoded="${uri#hook://file/}"
    encoded="${encoded%%#*}"  # strip fragment
    # Convert URL-safe base64 back to standard
    encoded="${encoded//-/+}"
    encoded="${encoded//_//}"
    python3 -c "
import base64, sys
s = sys.argv[1]
pad = (4 - len(s) % 4) % 4
print(base64.b64decode(s + '=' * pad).decode('utf-8', errors='replace'))
" "$encoded" 2>/dev/null || echo "$uri"
}

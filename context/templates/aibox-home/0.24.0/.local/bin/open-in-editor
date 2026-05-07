#!/bin/bash
# open-in-editor.sh — Open a file in the Vim editor pane/tab from Yazi.
#
# Behavior depends on layout:
# - dev/cowork-swap layouts: Vim is in the same tab, to the right
# - cowork layout: Vim is in the same tab, below
# - focus/browse/ai layouts: Vim is in a separate "editor" tab
#
# Set AIBOX_EDITOR_DIR to: right, down, or tab.
# When unset, the helper prefers an existing "editor" tab and falls back to
# the same-tab right-hand editor pane used by the dev layout.

file="${1:-}"
[ -z "$file" ] && exit 1

file="$(realpath "$file" 2>/dev/null || printf '%s' "$file")"

if [ -z "${ZELLIJ:-}" ] || ! command -v zellij >/dev/null 2>&1; then
    exec "${EDITOR:-vim}" "$file"
fi

editor_tab_start_delay="${AIBOX_EDITOR_TAB_START_DELAY:-0.5}"
editor_focus_delay="${AIBOX_EDITOR_FOCUS_DELAY:-0.35}"

has_editor_tab() {
    zellij action query-tab-names 2>/dev/null \
        | tr ' \t' '\n' \
        | grep -Fxq "editor"
}

dir="${AIBOX_EDITOR_DIR:-}"
if [ -z "$dir" ]; then
    if has_editor_tab; then
        dir="tab"
    else
        dir="right"
    fi
fi

vim_escape_path() {
    printf '%s' "$1" \
        | sed -e 's/\\/\\\\/g' \
              -e 's/ /\\ /g' \
              -e 's/	/\\	/g' \
              -e 's/|/\\|/g' \
              -e 's/%/\\%/g' \
              -e 's/#/\\#/g' \
              -e 's/!/\\!/g'
}

focused_pane_snapshot() {
    zellij action list-panes --json --all --command --state --tab 2>/dev/null \
        | tr '\n' ' ' \
        | sed 's/},[[:space:]]*{/}\n{/g' \
        | grep -E '"is_focused"[[:space:]]*:[[:space:]]*true' \
        | head -n 1
}

focused_pane_looks_like_editor() {
    snapshot="$(focused_pane_snapshot)"
    [ -n "$snapshot" ] || return 1
    snapshot="$(printf '%s' "$snapshot" | tr '[:upper:]' '[:lower:]')"

    case "$snapshot" in
        *vim*|*vim-loop*|*editor*) return 0 ;;
        *) return 1 ;;
    esac
}

send_to_vim() {
    sleep "$editor_focus_delay"
    if ! focused_pane_looks_like_editor; then
        if [ "${AIBOX_EDITOR_UNSAFE:-0}" != "1" ]; then
            echo "open-in-editor: refusing to send Vim commands to a non-editor Zellij pane" >&2
            echo "open-in-editor: set AIBOX_EDITOR_DIR=tab|right|down, or AIBOX_EDITOR_UNSAFE=1 to bypass" >&2
            exit 2
        fi
    fi

    vim_file="$(vim_escape_path "$file")"
    zellij action write 27
    sleep 0.05
    zellij action write-chars ":edit ${vim_file}"
    zellij action write 13
}

case "$dir" in
    tab)
        zellij action go-to-tab-name "editor"
        sleep "$editor_tab_start_delay"
        send_to_vim
        ;;
    down)
        zellij action move-focus down
        send_to_vim
        ;;
    *)
        zellij action move-focus right
        send_to_vim
        ;;
esac

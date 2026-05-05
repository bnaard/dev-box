#!/bin/bash
# open-in-editor.sh — Open a file in the Vim editor pane/tab from Yazi.
#
# Behavior depends on layout:
# - dev/cowork-swap layouts: Vim is in the same tab, to the right
# - cowork layout: Vim is in the same tab, below
# - focus/browse/ai layouts: Vim is in a separate "editor" tab
#
# Set AIBOX_EDITOR_DIR to: right (default), down, or tab.

file="${1:-}"
[ -z "$file" ] && exit 1

file="$(realpath "$file" 2>/dev/null || printf '%s' "$file")"

if [ -z "${ZELLIJ:-}" ] || ! command -v zellij >/dev/null 2>&1; then
    exec "${EDITOR:-vim}" "$file"
fi

dir="${AIBOX_EDITOR_DIR:-right}"
editor_tab_start_delay="${AIBOX_EDITOR_TAB_START_DELAY:-0.5}"
editor_focus_delay="${AIBOX_EDITOR_FOCUS_DELAY:-0.35}"

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

send_to_vim() {
    sleep "$editor_focus_delay"
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

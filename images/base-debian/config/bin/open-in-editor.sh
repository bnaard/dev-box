#!/bin/bash
# open-in-editor.sh — Open a file in the vim editor pane/tab from yazi.
#
# Behavior depends on layout:
# - dev layout: vim is in same tab, to the right → move-focus right
# - cowork layout: vim is in same tab, below → move-focus down
# - focus/browse/ai layouts: vim is in a separate tab → go-to-tab-name "editor"
#
# Set AIBOX_EDITOR_DIR to: right (default), down, or tab

file="$1"
[ -z "$file" ] && exit 1

file="$(realpath "$file" 2>/dev/null || echo "$file")"

dir="${AIBOX_EDITOR_DIR:-right}"
editor_tab_start_delay="${AIBOX_EDITOR_TAB_START_DELAY:-0.5}"

send_to_vim() {
    zellij action write 27
    sleep 0.05
    zellij action write-chars ":e ${file}"
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

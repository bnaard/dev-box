#!/bin/bash
# open-in-editor.sh — Open a file in the adjacent vim pane from yazi.
#
# Moves focus to the editor pane, sends :e <file> to vim, returns to yazi.
# Direction defaults to "right" (dev layout). Set DEVBOX_EDITOR_DIR=down
# for cowork layout.

file="$1"
[ -z "$file" ] && exit 1

file="$(realpath "$file" 2>/dev/null || echo "$file")"

dir="${DEVBOX_EDITOR_DIR:-right}"
case "$dir" in
    down) back="up" ;;
    *)    dir="right"; back="left" ;;
esac

zellij action move-focus "$dir"
zellij action write 27
sleep 0.05
zellij action write-chars ":e ${file}"
zellij action write 13
zellij action move-focus "$back"

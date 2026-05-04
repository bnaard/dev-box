#!/bin/bash
# open-in-editor.sh — Open a file from Yazi in Vim.

file="$1"
[ -z "$file" ] && exit 1

file="$(realpath "$file" 2>/dev/null || printf '%s' "$file")"

if [ -n "${ZELLIJ:-}" ] && command -v zellij >/dev/null 2>&1; then
    zellij action edit --in-place "$file" && exit 0
fi

exec "${EDITOR:-vim}" "$file"

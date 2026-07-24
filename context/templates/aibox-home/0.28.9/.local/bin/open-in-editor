#!/bin/bash
# open-in-editor.sh — Open a file in vim via a full-screen tmux popup.
#
# Bound to Yazi `e`. ':q' in vim closes the popup and returns focus to
# the originating yazi pane. There is no persistent vim pane in any
# aibox layout (DEC-20260508_1604-LuckySeal); every press of `e` starts
# a fresh, stateless vim session.
#
# When run outside tmux (or tmux is missing), falls back to a direct
# vim exec so file managers running in a plain terminal still work.

set -eu

file="${1:-}"
[ -z "$file" ] && exit 1
file="$(realpath "$file" 2>/dev/null || printf '%s' "$file")"

if [ -z "${TMUX:-}" ] || ! command -v tmux >/dev/null 2>&1; then
    exec "${EDITOR:-vim}" "$file"
fi

src_dir="$(dirname "$file")"
esc_file="$(printf '%q' "$file")"
exec tmux display-popup -E -w 100% -h 100% -d "$src_dir" "${EDITOR:-vim} $esc_file"

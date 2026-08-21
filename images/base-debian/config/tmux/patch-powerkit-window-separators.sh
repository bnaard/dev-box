#!/usr/bin/env bash
# Remove a stray literal `#` from tmux-powerkit window separator colour
# attributes.  `fg=#RRGGBB#` is invalid tmux syntax and leaks the surrounding
# status colour into rounded/powerline divider cells.
set -euo pipefail

target="${1:-}"
[[ -f "$target" ]] || {
    echo "Usage: patch-powerkit-window-separators.sh <windows.sh>" >&2
    exit 2
}

marker="AIBOX-FIX-WINDOW-SEPARATOR-COLOURS"
grep -q "$marker" "$target" && exit 0

matches="$(grep -o 'fg=%s#,' "$target" | wc -l | tr -d ' ')"
[[ "$matches" -gt 0 ]] || {
    echo "Expected malformed fg=%s#, attributes in $target" >&2
    exit 1
}

sed -i 's/fg=%s#,/fg=%s,/g' "$target"
printf '\n# %s\n' "$marker" >>"$target"

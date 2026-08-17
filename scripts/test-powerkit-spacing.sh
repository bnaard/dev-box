#!/usr/bin/env bash
# Deterministic format and isolated tmux/asciinema checks for aibox's
# side-aware tmux-powerkit plugin spacing patch.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
POWERKIT_SOURCE="${POWERKIT_SOURCE:-/usr/local/share/aibox/tmux/plugins/tmux-powerkit}"
RENDER_LIST="${HOME}/.local/bin/aibox-powerkit-render-list"
THEME_FILE="${HOME}/.config/tmux/aibox-powerkit-theme.sh"

[[ -d "$POWERKIT_SOURCE" ]] || { echo "PowerKit not installed: $POWERKIT_SOURCE" >&2; exit 1; }
[[ -x "$RENDER_LIST" ]] || { echo "Missing renderer: $RENDER_LIST" >&2; exit 1; }
[[ -r "$THEME_FILE" ]] || { echo "Missing aibox PowerKit theme: $THEME_FILE" >&2; exit 1; }

tmpdir="$(mktemp -d)"
socket_label="aibox-spacing-$RANDOM-$$"
session="spacing"
cleanup() {
    tmux -L "$socket_label" kill-server 2>/dev/null || true
    rm -rf -- "$tmpdir"
}
trap cleanup EXIT

cp -R "$POWERKIT_SOURCE" "$tmpdir/powerkit"
"${PROJECT_ROOT}/images/base-debian/config/tmux/patch-powerkit-plugin-spacing.sh" \
    "$tmpdir/powerkit/src/renderer/segment_builder.sh"
bash -n "$tmpdir/powerkit/src/renderer/segment_builder.sh"

tmux -L "$socket_label" new-session -d -s "$session" -x 160 -y 12
tmux -L "$socket_label" set-option -g status on
tmux -L "$socket_label" set-option -g status-interval 1
tmux -L "$socket_label" set-option -g status-style 'bg=#313244,fg=#cdd6f4'
tmux -L "$socket_label" set-option -g @powerkit_theme custom
tmux -L "$socket_label" set-option -g @powerkit_custom_theme_path "$THEME_FILE"
tmux -L "$socket_label" set-option -g @powerkit_separator_style normal
tmux -L "$socket_label" set-option -g @powerkit_edge_separator_style rounded
tmux -L "$socket_label" set-option -g @powerkit_elements_spacing both
tmux -L "$socket_label" set-option -g @powerkit_transparent false

socket_path="$(tmux -L "$socket_label" display-message -p '#{socket_path}')"
tmux -L "$socket_label" set-environment -g POWERKIT_ROOT "$tmpdir/powerkit"
tmux -L "$socket_label" set-environment -g AIBOX_TMUX_SOCKET "$socket_path"
tmux_env=(
    "TMUX=${socket_path},0,0"
    "AIBOX_TMUX_SOCKET=${socket_path}"
    "POWERKIT_ROOT=${tmpdir}/powerkit"
    "AIBOX_POWERKIT_REFRESH_CACHE=1"
)

env "${tmux_env[@]}" "$RENDER_LIST" left uptime,datetime > "$tmpdir/left.format"
env "${tmux_env[@]}" "$RENDER_LIST" right uptime,datetime > "$tmpdir/right.format"

python3 - "$tmpdir/left.format" "$tmpdir/right.format" <<'PY'
import re
import sys
from pathlib import Path

left = Path(sys.argv[1]).read_text()
right = Path(sys.argv[2]).read_text()
color = r"#[0-9A-Fa-f]{6}|default"

left_boundary = re.search(
    rf"#\[fg=(?P<previous>{color}),bg=(?P<gap>{color})\](?P<glyph>.)#\[none\]"
    rf"#\[range=user\|datetime\]#\[fg=(?P=gap),bg=(?P<next>{color})\](?P=glyph)#\[none\]",
    left,
)
assert left_boundary, f"left boundary is not previous -> gap -> next:\n{left}"
assert left_boundary["previous"] != left_boundary["gap"]
assert left_boundary["next"] != left_boundary["gap"]

right_boundary = re.search(
    rf"#\[fg=(?P<gap>{color}),bg=(?P<previous>{color})\](?P<glyph>.)#\[none\]"
    rf"#\[range=user\|datetime\]#\[fg=(?P<next>{color}),bg=(?P=gap)\](?P=glyph)#\[none\]",
    right,
)
assert right_boundary, f"right boundary is not previous -> gap -> next:\n{right}"
assert right_boundary["previous"] != right_boundary["gap"]
assert right_boundary["next"] != right_boundary["gap"]
PY

echo "PowerKit left/right separator format invariants passed"

if command -v asciinema >/dev/null 2>&1; then
    cast="${AIBOX_POWERKIT_SPACING_CAST:-${PROJECT_ROOT}/tmp/powerkit-spacing.cast}"
    mkdir -p "$(dirname "$cast")"
    left_cmd="POWERKIT_ROOT=${tmpdir}/powerkit AIBOX_TMUX_SOCKET=${socket_path} AIBOX_POWERKIT_REFRESH_CACHE=1 ${RENDER_LIST} left uptime,datetime"
    right_cmd="POWERKIT_ROOT=${tmpdir}/powerkit AIBOX_TMUX_SOCKET=${socket_path} AIBOX_POWERKIT_REFRESH_CACHE=1 ${RENDER_LIST} right uptime,datetime"
    tmux -L "$socket_label" set-option -g 'status-format[0]' \
        "#[align=left]#(${left_cmd})#[align=right]#(${right_cmd})"
    tmux -L "$socket_label" refresh-client -S 2>/dev/null || true

    (sleep 3; tmux -L "$socket_label" kill-session -t "$session" 2>/dev/null || true) &
    asciinema rec --cols 160 --rows 12 --idle-time-limit 1 --overwrite \
        -c "tmux -L ${socket_label} attach-session -t ${session}" "$cast" >/dev/null 2>&1 || true
    [[ -s "$cast" ]] || { echo "Asciinema spacing cast was not created" >&2; exit 1; }
    echo "PowerKit spacing visual cast: $cast"
fi

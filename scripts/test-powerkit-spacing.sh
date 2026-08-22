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
"${PROJECT_ROOT}/images/base-debian/config/tmux/patch-powerkit-window-separators.sh" \
    "$tmpdir/powerkit/src/renderer/entities/windows.sh"
bash -n "$tmpdir/powerkit/src/renderer/segment_builder.sh"
bash -n "$tmpdir/powerkit/src/renderer/entities/windows.sh"

# Upstream may add legitimate conditional branches without changing the
# separator contract. Reproduce the pinned 6ac71f0 shape (one extra escaped
# branch) and ensure the downstream compatibility patch remains structural
# instead of rejecting the file by global occurrence count.
expanded_windows="$tmpdir/windows-expanded.sh"
cp "$tmpdir/powerkit/src/renderer/entities/windows.sh" "$expanded_windows"
sed -i '0,/if \[\[ "$side" == "left" || "$side" == "center" \]\]; then/{
    /if \[\[ "$side" == "left" || "$side" == "center" \]\]; then/i\
    if false; then printf '\''#{?expanded,#[none]#[fg=%s#,bg=%s]x,}'\'' x x; fi
}' "$expanded_windows"
"${PROJECT_ROOT}/images/base-debian/config/tmux/patch-powerkit-window-separators.sh" \
    "$expanded_windows"
bash -n "$expanded_windows"
[[ "$(grep -o 'fg=%s#,bg=%s' "$expanded_windows" | wc -l | tr -d ' ')" -gt 8 ]] || {
    echo "Expanded PowerKit fixture did not exercise the former exact-count failure" >&2
    exit 1
}

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
tmux -L "$socket_label" new-window -d -t "$session" -n second

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

# Load the patched renderer into a real isolated tmux server. A comma inside a
# conditional style must be written as `#,`; otherwise tmux treats it as the
# branch delimiter, drops the separator, and renders the remainder literally.
env "TMUX=$(tmux -L "$socket_label" display-message -p '#{socket_path}'),0,0" \
    POWERKIT_ROOT="$tmpdir/powerkit" \
    bash -c '. "$POWERKIT_ROOT/src/core/bootstrap.sh"; powerkit_bootstrap; . "$POWERKIT_ROOT/src/renderer/entities/windows.sh"; windows_configure left'

window_format="$(tmux -L "$socket_label" show-option -gv window-status-format)"
current_format="$(tmux -L "$socket_label" show-option -gv window-status-current-format)"
[[ "$window_format" == *'#,bg='* && "$current_format" == *'#,bg='* ]] || {
    echo "Window formats lost their escaped conditional commas" >&2
    exit 1
}

session_format="$(env "TMUX=$(tmux -L "$socket_label" display-message -p '#{socket_path}'),0,0" \
    POWERKIT_ROOT="$tmpdir/powerkit" \
    "$HOME/.local/bin/aibox-powerkit-render-session")"

python3 - "$session_format" "$window_format" "$current_format" <<'PY'
import re
import sys

session = sys.argv[1]
formats = (("inactive", sys.argv[2]), ("active", sys.argv[3]))

session_end = re.search(r"#\[fg=.+,bg=(#[0-9A-Fa-f]{6})\](.)$", session)
assert session_end, f"session lacks a colored closing edge: {session}"
status_bg, session_glyph = session_end.groups()
assert session_glyph == "", f"session edge is not rounded: {session}"

for label, value in formats:
    assert value.count("#[none]#[fg=") >= 3, (
        f"{label} separators do not reset inherited text attributes: {value}"
    )
    colors = re.findall(r"#\[fg=(#[0-9A-Fa-f]{6})#?,bg=(#[0-9A-Fa-f]{6})(?:,[^]]+)?\]", value)
    assert len(colors) >= 5, f"{label} window format lacks expected color segments: {value}"
    incoming, index, index_arrow, name, outgoing = colors[:5]
    assert incoming[0] == status_bg == outgoing[1], (
        f"{label} status gap is not session -> incoming peak / outgoing background: {value}"
    )
    assert incoming[1] == index[1] == index_arrow[0], (
        f"{label} light index background/arrow sequence is discontinuous: {value}"
    )
    assert index_arrow[1] == name[1] == outgoing[0], (
        f"{label} window-name background/arrow sequence is discontinuous: {value}"
    )
PY

expanded_windows="$(tmux -L "$socket_label" display-message -p '#{W:#{T:window-status-format},#{T:window-status-current-format}}')"
visible_windows="$(printf '%s' "$expanded_windows" | sed 's/#\[[^]]*\]//g')"
[[ "$visible_windows" != *'bg=#'* && "$visible_windows" != *',}'* ]] || {
    echo "tmux leaked a conditional colour branch: $expanded_windows" >&2
    exit 1
}
[[ "$visible_windows" == *'1'* && "$visible_windows" == *'2'* && "$visible_windows" == *''* ]] || {
    echo "tmux did not render both spaced window segments: $expanded_windows" >&2
    exit 1
}

echo "PowerKit window conditional separator regression passed"

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

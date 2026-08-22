#!/usr/bin/env bash
# Preserve tmux's escaped commas inside conditional window formats.
#
# PowerKit emits attributes such as `#[fg=%s#,bg=%s]` inside `#{?...,...,...}`.
# The `#` escapes the following comma for tmux's format parser; it is not part
# of the foreground colour.  Repair images produced by the former aibox patch,
# which removed that escape and caused text such as `bg=#21252B]` to leak into
# the status bar. Preserve PowerKit's intentional two-chevron spacing model:
# the outgoing peak uses the window-name background and the incoming peak uses
# the status-bar background before transitioning into the next index cell.
# Reset text attributes at every separator so an inactive window's `dim` style
# cannot darken an otherwise correct separator foreground color.
set -euo pipefail

target="${1:-}"
[[ -f "$target" ]] || {
    echo "Usage: patch-powerkit-window-separators.sh <windows.sh>" >&2
    exit 2
}

marker="AIBOX-FIX-WINDOW-SEPARATOR-CONDITIONAL-COMMAS"

# Only attributes embedded in conditional branches need `#,`. Plain tmux
# attributes elsewhere in the renderer must retain their ordinary comma.
sed -i \
    -e "/printf '%s#\\[fg=%s,bg=%s/ s/fg=%s,bg=%s/fg=%s#,bg=%s/g" \
    -e "/printf '#{?/ s/fg=%s,bg=%s/fg=%s#,bg=%s/g" \
    -e '/_windows_build_separator()/,/^}/ s/local side="\$1" index_bg="\$2" previous_bg="\$3" content_bg="\$4"/local side="\$1" index_bg="\$2" previous_bg="\$3"/' \
    -e '/_windows_build_separator()/,/^}/ s/spacing_fg="\$content_bg"/spacing_fg=\$(get_color "statusbar-bg")/' \
    -e 's/_windows_build_separator "\$side" "\$first_segment_bg" "\$previous_bg" "\$content_bg")/_windows_build_separator "\$side" "\$first_segment_bg" "\$previous_bg")/' \
    -e '/_windows_build_separator()/,/^}/ s/#\[none\]#\[fg=/#[fg=/g' \
    -e '/_windows_build_separator()/,/^}/ s/#\[fg=/#[none]#[fg=/g' \
    -e '/_windows_build_index_sep()/,/^}/ s/#\[none\]#\[fg=/#[fg=/g' \
    -e '/_windows_build_index_sep()/,/^}/ s/#\[fg=/#[none]#[fg=/g' \
    -e '/_windows_build_spacing()/,/^}/ s/#\[none\]#\[fg=/#[fg=/g' \
    -e '/_windows_build_spacing()/,/^}/ s/#\[fg=/#[none]#[fg=/g' \
    -e '/AIBOX-FIX-WINDOW-SEPARATOR-COLOURS/d' \
    "$target"

escaped="$(grep -o 'fg=%s#,bg=%s' "$target" | wc -l | tr -d ' ')"
[[ "$escaped" -eq 8 ]] || {
    echo "Expected 8 escaped conditional colour attributes in $target, found $escaped" >&2
    exit 1
}

grep -q 'local side="$1" index_bg="$2" previous_bg="$3"$' "$target" || {
    echo "Window separator helper signature is not canonical" >&2
    exit 1
}
[[ "$(grep -c '_windows_build_separator "$side" "$first_segment_bg" "$previous_bg")' "$target")" -eq 2 ]] || {
    echo "Expected both window format builders to use the canonical separator call" >&2
    exit 1
}
[[ "$(grep -o '#\[none\]#\[fg=' "$target" | wc -l | tr -d ' ')" -eq 11 ]] || {
    echo "Window separator glyphs do not consistently reset inherited styles" >&2
    exit 1
}

grep -q "$marker" "$target" || printf '\n# %s\n' "$marker" >>"$target"

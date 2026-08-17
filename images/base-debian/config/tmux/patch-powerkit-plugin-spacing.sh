#!/usr/bin/env bash
# Patch tmux-powerkit plugin gaps to use the same side-aware two-chevron
# transition as its window renderer.
set -euo pipefail

target="${1:-}"
[[ -f "$target" ]] || {
    echo "Usage: patch-powerkit-plugin-spacing.sh <segment_builder.sh>" >&2
    exit 2
}

marker="AIBOX-FIX-SIDE-AWARE-PLUGIN-SPACING"
grep -q "$marker" "$target" && exit 0

needle='                output+="#[fg=${current_spacing_fg},bg=${prev_bg}]${spacing_sep}#[none]"'
legacy_needle='                output+="#[fg=${current_spacing_fg},bg=${prev_bg}]${spacing_sep}" # AIBOX-FIX-GH-SEPARATOR-MARKER'
legacy_gap='                output+="#[fg=${current_spacing_fg},bg=${current_spacing_bg}] #[none]"'
replacement='                if [[ "$side" == "left" ]]; then
                    # Right-facing: previous segment -> status-bar gap.
                    output+="#[fg=${prev_bg},bg=${current_spacing_bg}]${spacing_sep}#[none]"
                else
                    # Left-facing: status-bar gap <- previous segment.
                    output+="#[fg=${current_spacing_fg},bg=${prev_bg}]${spacing_sep}#[none]"
                fi # AIBOX-FIX-SIDE-AWARE-PLUGIN-SPACING'

tmp="${target}.aibox-spacing.$$"
trap 'rm -f -- "$tmp"' EXIT

matched=0
while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" == "$needle" || "$line" == "$legacy_needle" ]]; then
        printf '%s\n' "$replacement" >> "$tmp"
        matched=$((matched + 1))
        if [[ "$line" == "$legacy_needle" ]]; then
            IFS= read -r line || {
                echo "Legacy spacing patch is missing its gap line" >&2
                exit 1
            }
            [[ "$line" == "$legacy_gap" ]] || {
                echo "Legacy spacing patch has an unexpected gap line" >&2
                exit 1
            }
        fi
    else
        printf '%s\n' "$line" >> "$tmp"
    fi
done < "$target"

[[ "$matched" -eq 1 ]] || {
    echo "Expected exactly one upstream plugin-spacing line, found $matched" >&2
    exit 1
}

chmod --reference="$target" "$tmp"
mv -- "$tmp" "$target"
trap - EXIT

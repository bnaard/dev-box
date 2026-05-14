#!/bin/bash
# aibox-preview.sh — Full-pane preview helper for Yazi.
#
# Design contract: the rendering that `p` produces should *match* what
# Yazi's preview pane shows for the same file, just at full-pane size.
# For markdown / rst / json / ipynb that means using the same Python
# `rich` pipeline the `rich-preview.yazi` plugin uses — so the user
# never sees a stylistic jump between "small preview" and "big preview".
# For code/data, `bat` is the equivalent of Yazi's built-in code
# previewer. For PDFs, the `pdf-watch` helper is dispatched.

set -euo pipefail

mode="${AIBOX_PREVIEW_MODE:-auto}"
if [ "$#" -ge 2 ]; then
    mode="$1"
    shift
fi

file="${1:-}"
if [ -z "$file" ]; then
    echo "aibox-preview: no file selected" >&2
    exit 1
fi

if [ ! -e "$file" ]; then
    echo "aibox-preview: file not found: $file" >&2
    exit 1
fi

ext="${file##*.}"
if [ "$ext" = "$file" ]; then
    ext=""
fi
ext="$(printf '%s' "$ext" | tr '[:upper:]' '[:lower:]')"

pager_flags=(-R)
if less --help 2>&1 | grep -q -- '--mouse'; then
    pager_flags=(--mouse --wheel-lines=3 "${pager_flags[@]}")
fi

page_ansi() {
    "${PAGER:-less}" "${pager_flags[@]}"
}

page_plain() {
    "${PAGER:-less}" "${pager_flags[@]}" "$file"
}

# Probe whether `python3 -c 'import rich'` succeeds. Cached in the env
# so successive calls in the same shell are free.
_aibox_preview_have_rich() {
    if [ -n "${AIBOX_PREVIEW_HAVE_RICH:-}" ]; then
        [ "$AIBOX_PREVIEW_HAVE_RICH" = "1" ]
        return $?
    fi
    if command -v python3 >/dev/null 2>&1 \
       && python3 -c 'import rich' >/dev/null 2>&1; then
        export AIBOX_PREVIEW_HAVE_RICH=1
        return 0
    fi
    export AIBOX_PREVIEW_HAVE_RICH=0
    return 1
}

# Render via the same Python rich pipeline as the rich-preview.yazi
# plugin. Reads $file, prints ANSI to stdout.
preview_rich() {
    local cols="${COLUMNS:-0}"
    if [ "$cols" -le 0 ]; then
        # tput is the most portable column probe; fallback to a sensible 100.
        cols="$(tput cols 2>/dev/null || echo 100)"
    fi
    python3 - "$file" "$cols" <<'PY' | page_ansi
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
width = int(sys.argv[2])
text = path.read_text(errors="replace")

from rich.console import Console
from rich.markdown import Markdown
from rich.syntax import Syntax

console = Console(width=width, force_terminal=True, color_system="truecolor", soft_wrap=False)
if path.suffix.lower() in {".md", ".markdown"}:
    console.print(Markdown(text))
else:
    language = path.suffix.lstrip(".") or "text"
    console.print(Syntax(text, language, theme="ansi_dark", line_numbers=True, word_wrap=False))
PY
}

preview_markdown() {
    # Prefer the Python rich pipeline so the full-pane preview matches
    # Yazi's in-pane rich-preview output. Falls back to `glow` (still a
    # good rich renderer) and then `bat` and finally raw `less` so the
    # command always produces *some* output even on a minimal image.
    if _aibox_preview_have_rich; then
        preview_rich || true
    elif command -v glow >/dev/null 2>&1; then
        glow -s "${AIBOX_GLOW_STYLE:-dark}" "$file" | page_ansi || true
    elif command -v bat >/dev/null 2>&1; then
        bat --paging=never --style=full --color=always --language=md "$file" | page_ansi || true
    else
        page_plain
    fi
}

preview_code() {
    if command -v bat >/dev/null 2>&1; then
        bat --paging=never --style=full --color=always "$file" | page_ansi || true
    else
        page_plain
    fi
}

preview_pdf() {
    if command -v pdf-watch >/dev/null 2>&1; then
        pdf-watch "$file"
    elif [ -x "$HOME/.local/bin/pdf-watch" ]; then
        "$HOME/.local/bin/pdf-watch" "$file"
    else
        echo "aibox-preview: PDF preview requires pdf-watch" >&2
        exit 1
    fi
}

# Image preview via chafa (terminal images). Matches what the Yazi
# image previewer shows when running in chafa-fallback mode (e.g. on
# terminals without sixel/kitty support).
preview_image() {
    if command -v chafa >/dev/null 2>&1; then
        chafa --animate=off --colors=truecolor --size="${COLUMNS:-100}x${LINES:-30}" "$file" | page_ansi || true
    elif command -v bat >/dev/null 2>&1; then
        echo "aibox-preview: chafa not installed — falling back to file metadata"
        bat --paging=never --style=full --color=always --language=md - <<META | page_ansi || true
# $(basename "$file")

$(stat -c '%n  %s bytes  %y' "$file" 2>/dev/null || file "$file")
META
    else
        page_plain
    fi
}

case "$mode" in
    markdown|md|rst|rich)
        preview_markdown
        ;;
    json|ipynb)
        # rich Syntax for these matches Yazi's rich-preview routing.
        if _aibox_preview_have_rich; then
            preview_rich
        else
            preview_code
        fi
        ;;
    code|text|txt)
        preview_code
        ;;
    pdf)
        preview_pdf
        ;;
    image|img)
        preview_image
        ;;
    auto)
        case "$ext" in
            md|markdown|mdown|mkd|rst)
                preview_markdown
                ;;
            json|ipynb)
                if _aibox_preview_have_rich; then
                    preview_rich
                else
                    preview_code
                fi
                ;;
            pdf)
                preview_pdf
                ;;
            png|jpg|jpeg|gif|webp|bmp|tiff|tif|svg)
                preview_image
                ;;
            *)
                preview_code
                ;;
        esac
        ;;
    *)
        echo "aibox-preview: unknown mode: $mode" >&2
        exit 1
        ;;
esac

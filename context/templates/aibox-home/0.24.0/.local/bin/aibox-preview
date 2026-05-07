#!/bin/bash
# aibox-preview.sh — Full-pane preview helper for Yazi.

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

page_plain() {
    "${PAGER:-less}" "$file"
}

preview_markdown() {
    if command -v glow >/dev/null 2>&1; then
        glow -p -s "${AIBOX_GLOW_STYLE:-dark}" "$file"
    elif command -v bat >/dev/null 2>&1; then
        bat --paging=always --style=full --color=always --language=md "$file"
    else
        page_plain
    fi
}

preview_code() {
    if command -v bat >/dev/null 2>&1; then
        bat --paging=always --style=full --color=always "$file"
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

case "$mode" in
    markdown|md)
        preview_markdown
        ;;
    code|text|txt)
        preview_code
        ;;
    pdf)
        preview_pdf
        ;;
    auto)
        case "$ext" in
            md|markdown|mdown|mkd)
                preview_markdown
                ;;
            pdf)
                preview_pdf
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

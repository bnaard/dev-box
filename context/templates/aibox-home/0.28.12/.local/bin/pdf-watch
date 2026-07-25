#!/usr/bin/env bash
set -euo pipefail

file="${1:-}"
page="${2:-1}"

if [[ -z "$file" ]]; then
  echo "Usage: pdf-watch <file.pdf> [page]" >&2
  exit 2
fi

if [[ ! -f "$file" ]]; then
  echo "pdf-watch: file not found: $file" >&2
  exit 1
fi

if [[ "${file,,}" != *.pdf ]]; then
  echo "pdf-watch: not a PDF: $file" >&2
  exit 1
fi

for cmd in mutool timg entr; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "pdf-watch: missing required command: $cmd" >&2
    exit 1
  fi
done

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/aibox-pdf-watch.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT

render() {
  local output="$tmp_dir/page.png"
  if mutool draw -q -r 144 -o "$output" "$file" "$page" >/dev/null; then
    timg --clear "$output"
  else
    echo "pdf-watch: waiting for a readable PDF..." >&2
  fi
}

export file page tmp_dir
export -f render

render
printf '%s\n' "$file" | entr -c bash -lc render

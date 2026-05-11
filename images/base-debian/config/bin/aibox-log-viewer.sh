#!/usr/bin/env bash
set -euo pipefail

workspace="${AIBOX_WORKSPACE:-/workspace}"
log_path="${AIBOX_LOG_PATH:-${workspace}/.aibox/aibox.log}"
rotated_path="${AIBOX_LOG_ROTATED_PATH:-${log_path}.1}"

logs=()
if [ -r "$rotated_path" ]; then
  logs+=("$rotated_path")
fi
if [ -r "$log_path" ]; then
  logs+=("$log_path")
fi

if [ "${#logs[@]}" -eq 0 ]; then
  printf 'No aibox log found at %s\n' "$log_path" >&2
  sleep 2
  exit 1
fi

if command -v lnav >/dev/null 2>&1; then
  exec lnav -N -q -c ':goto 100%' "${logs[@]}"
fi

less_flags=(-R +G)
if less --help 2>&1 | grep -q -- '--mouse'; then
  less_flags=(--mouse --wheel-lines=3 "${less_flags[@]}")
fi

if [ "${#logs[@]}" -eq 1 ]; then
  exec less "${less_flags[@]}" "${logs[0]}"
fi

tmp="$(mktemp -t aibox-log-viewer.XXXXXX)"
trap 'rm -f "$tmp"' EXIT
cat "${logs[@]}" > "$tmp"
less "${less_flags[@]}" "$tmp"

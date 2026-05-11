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

if [ "${AIBOX_LOG_VIEWER:-pretty}" = "lnav" ] && command -v lnav >/dev/null 2>&1; then
  exec lnav -N -q -c ':goto 100%' "${logs[@]}"
fi

less_flags=(-R +G)
if less --help 2>&1 | grep -q -- '--mouse'; then
  less_flags=(--mouse --wheel-lines=3 "${less_flags[@]}")
fi

if command -v jq >/dev/null 2>&1; then
  tmp="$(mktemp -t aibox-log-viewer.XXXXXX)"
  trap 'rm -f "$tmp"' EXIT
  jq -r -n '
    def value($v): ($v // "-") | tostring;
    def session_id($row):
      $row.runtime_session_id // $row.container_id // $row.container // $row.runtime_started_at // "legacy/no-session";
    def session_header($row):
      "\n\u001b[2m---- session: \(session_id($row)) started=\(value($row.runtime_started_at)) ----\u001b[0m";
    def inferred_level($row):
      ($row.level // (if (($row.exit_code // 0) | tonumber) == 0 then "info" else "error" end))
      | ascii_upcase;
    def painted_level($row):
      inferred_level($row) as $level
      | if $level == "ERROR" then "\u001b[31;1mERROR\u001b[0m"
        elif $level == "WARN" or $level == "WARNING" then "\u001b[33;1mWARN \u001b[0m"
        else "\u001b[36mINFO \u001b[0m"
        end;
    def render($row):
      "\(value($row.ts // $row.timestamp))  \(painted_level($row))  \(value($row.cmd // $row.command))  v\(value($row.version))  \(value($row.duration_ms))ms  exit=\(value($row.exit_code))  \(value($row.msg // $row.message))";
    foreach inputs as $row ({first: true, last: null};
      (session_id($row)) as $sid
      | .emit_header = (.first or .last != $sid)
      | .first = false
      | .last = $sid;
      (if .emit_header then session_header($row) else empty end), render($row)
    )
  ' "${logs[@]}" > "$tmp"
  less "${less_flags[@]}" "$tmp"
  exit $?
fi

if command -v lnav >/dev/null 2>&1; then
  exec lnav -N -q -c ':goto 100%' "${logs[@]}"
fi

if [ "${#logs[@]}" -eq 1 ]; then
  exec less "${less_flags[@]}" "${logs[0]}"
fi

tmp="$(mktemp -t aibox-log-viewer.XXXXXX)"
trap 'rm -f "$tmp"' EXIT
cat "${logs[@]}" > "$tmp"
less "${less_flags[@]}" "$tmp"

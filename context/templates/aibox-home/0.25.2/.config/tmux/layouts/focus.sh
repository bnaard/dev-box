#!/usr/bin/env bash
set -euo pipefail

session="${AIBOX_TMUX_SESSION:-aibox}"
workspace="${AIBOX_WORKSPACE:-/workspace}"
config="${AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}"

if tmux -f "$config" has-session -t "$session" 2>/dev/null; then
  exec tmux -f "$config" attach-session -t "$session"
fi

tool_or_shell() {
  local tool="$1"
  printf "bash -lc 'if command -v %q >/dev/null 2>&1; then %q; fi; exec bash'" "$tool" "$tool"
}

tmux -f "$config" new-session -d -s "$session" -n focus -c "$workspace" "$(tool_or_shell codex)"
tmux new-window -t "$session:" -n editor -c "$workspace" "$(tool_or_shell vim)"
tmux select-window -t "$session:focus" 2>/dev/null || true
exec tmux -f "$config" attach-session -t "$session"

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

tmux -f "$config" new-session -d -s "$session" -n cowork -c "$workspace" "$(tool_or_shell vim)"
editor_pane="$(tmux display-message -p -t "$session:cowork" '#{pane_id}')"
agent_pane="$(tmux split-window -t "$session:cowork" -h -p 50 -P -F '#{pane_id}' -c "$workspace" "$(tool_or_shell codex)")"
files_pane="$(tmux split-window -t "$editor_pane" -v -p 35 -P -F '#{pane_id}' -c "$workspace" "$(tool_or_shell yazi)")"
tmux select-pane -t "$editor_pane"
tmux select-window -t "$session:cowork" 2>/dev/null || true
exec tmux -f "$config" attach-session -t "$session"

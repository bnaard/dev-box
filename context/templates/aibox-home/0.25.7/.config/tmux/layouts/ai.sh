#!/usr/bin/env bash
set -euo pipefail

session="${AIBOX_TMUX_SESSION:-aibox}"
workspace="${AIBOX_WORKSPACE:-/workspace}"
config="${AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}"
socket="${AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}"
mkdir -p "$(dirname "$socket")"

if tmux -S "$socket" -f "$config" has-session -t "$session" 2>/dev/null; then
  exec tmux -S "$socket" -f "$config" attach-session -t "$session"
fi

tool_or_shell() {
  local tool="$1"
  if [[ "$tool" == "yazi" ]]; then
    printf "bash -lc 'for _ in {1..50}; do tmux -S %q list-clients -t %q >/dev/null 2>&1 && break; sleep 0.1; done; if command -v yazi >/dev/null 2>&1; then exec yazi; fi; exec bash'" "$socket" "$session"
    return
  fi
  printf "bash -lc 'if command -v %q >/dev/null 2>&1; then %q; fi; exec bash'" "$tool" "$tool"
}

tmux -S "$socket" -f "$config" new-session -d -s "$session" -n ai -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:ai" '#{pane_id}')"
split_flag="-${AIBOX_LAYOUT_AGENT_SPLIT:-h}"
split_ratio="${AIBOX_LAYOUT_AGENT_RATIO:-50}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:ai" "$split_flag" -p "$split_ratio" -P -F '#{pane_id}' -c "$workspace" "$(tool_or_shell claude)")"
agent_pane_2="$(tmux -S "$socket" split-window -t "${agent_pane}" -v -p 20 -P -F '#{pane_id}' -c "$workspace" "$(tool_or_shell codex)")"
tmux -S "$socket" new-window -t "$session:" -n shell -c "$workspace" "bash"
tmux -S "$socket" select-window -t "$session:ai"
tmux -S "$socket" select-pane -t "$files_pane"
tmux -S "$socket" select-window -t "$session:ai" 2>/dev/null || true
exec tmux -S "$socket" -f "$config" attach-session -t "$session"

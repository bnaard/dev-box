#!/usr/bin/env bash
set -euo pipefail

session="${AIBOX_TMUX_SESSION:-aibox}"
workspace="${AIBOX_WORKSPACE:-/workspace}"
config="${AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}"

if tmux -f "$config" has-session -t "$session" 2>/dev/null; then
  exec tmux -f "$config" attach-session -t "$session"
fi

tmux -f "$config" new-session -d -s "$session" -n focus -c "$workspace" "codex"
tmux new-window -t "$session:" -n editor -c "$workspace" "vim"
tmux select-window -t "$session:1"
exec tmux -f "$config" attach-session -t "$session"

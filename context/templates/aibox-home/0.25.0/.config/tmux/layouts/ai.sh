#!/usr/bin/env bash
set -euo pipefail

session="${AIBOX_TMUX_SESSION:-aibox}"
workspace="${AIBOX_WORKSPACE:-/workspace}"
config="${AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}"

if tmux -f "$config" has-session -t "$session" 2>/dev/null; then
  exec tmux -f "$config" attach-session -t "$session"
fi

tmux -f "$config" new-session -d -s "$session" -n ai -c "$workspace" "yazi"
tmux split-window -t "$session:ai" -h -p 50 -c "$workspace" "codex"
tmux new-window -t "$session:" -n editor -c "$workspace" "vim"
tmux new-window -t "$session:" -n shell -c "$workspace" "bash"
tmux select-window -t "$session:1"
exec tmux -f "$config" attach-session -t "$session"

#!/usr/bin/env bash
set -euo pipefail

session="${AIBOX_TMUX_SESSION:-aibox}"
workspace="${AIBOX_WORKSPACE:-/workspace}"
config="${AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}"

if tmux -f "$config" has-session -t "$session" 2>/dev/null; then
  exec tmux -f "$config" attach-session -t "$session"
fi

tmux -f "$config" new-session -d -s "$session" -n cowork-swap -c "$workspace" "yazi"
tmux split-window -t "$session:cowork-swap" -v -p 45 -c "$workspace" "codex"
tmux split-window -t "$session:cowork-swap" -h -p 60 -c "$workspace" "vim"
tmux select-pane -t "$session:cowork-swap.3"
tmux select-window -t "$session:1"
exec tmux -f "$config" attach-session -t "$session"
